use crate::field::{format_value, AllowedValues, FieldValue, FieldValues};
use actix_web::{error::JsonPayloadError, http::StatusCode, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ObjectKind {
    #[serde(rename = "save")]
    Save,
    #[serde(rename = "solar-system")]
    SolarSystem,
    #[serde(rename = "star")]
    Star,
    #[serde(rename = "planet")]
    Planet,
    #[serde(rename = "planet-type")]
    PlanetType,
    #[serde(rename = "item")]
    Item,
    #[serde(rename = "item-recipe")]
    ItemRecipe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<ObjectKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<FieldValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<FieldValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<AllowedValues>,
}

#[derive(Error, Debug)]
pub enum TrackerError {
    #[error("No {0} exists with {1}.")]
    NotFound(ObjectKind, FieldValues),
    #[error("Unexpectedly did not find {0} with {1}.")]
    UnexpectedNotFound(ObjectKind, FieldValues),
    #[error("A {0} with the {1} already exists.")]
    Duplicate(ObjectKind, FieldValues),
    #[error("The value `{}` for the field {} is invalid. {1}", format_value(&.0.value), .0.name)]
    InvalidFieldValue(FieldValue, AllowedValues),
    #[error("Missing required field {0}. {1}")]
    MissingRequiredField(String, AllowedValues),
    #[error("Another transaction has already updated the {0} with {1}. Please try again.")]
    ConcurrentUpdate(ObjectKind, FieldValues),
    #[error("{0}")]
    SqlError(#[from] sqlx::Error),
    #[error("{0}")]
    JsonError(#[from] actix_web::error::JsonPayloadError),
    #[error("{0}")]
    QueryStringError(#[from] actix_web::error::QueryPayloadError),
    #[error("{0}")]
    PathError(#[from] actix_web::error::PathError),
}

pub type Result<T> = std::result::Result<T, TrackerError>;

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Item => "item",
                Self::ItemRecipe => "item recipe",
                Self::Save => "save",
                Self::SolarSystem => "solar system",
                Self::Planet => "planet",
                Self::Star => "star",
                Self::PlanetType => "planet type",
            }
        )
    }
}

impl TrackerError {
    pub fn not_found<K: Into<FieldValues>>(object: ObjectKind, keys: K) -> Self {
        Self::NotFound(object, keys.into())
    }

    pub fn duplicate<K: Into<FieldValues>>(object: ObjectKind, keys: K) -> Self {
        Self::Duplicate(object, keys.into())
    }

    pub fn concurrent_update<K: Into<FieldValues>>(object: ObjectKind, keys: K) -> Self {
        Self::ConcurrentUpdate(object, keys.into())
    }

    pub fn invalid_field(field: FieldValue, allowed_values: AllowedValues) -> Self {
        Self::InvalidFieldValue(field, allowed_values)
    }

    pub fn missing_field<F: Into<String>>(field: F, allowed_values: AllowedValues) -> Self {
        Self::MissingRequiredField(field.into(), allowed_values)
    }

    pub fn is_internal_server_error(&self) -> bool {
        match self {
            Self::UnexpectedNotFound(..) | Self::SqlError(..) => true,
            Self::JsonError(json_err) => matches!(json_err, JsonPayloadError::Serialize(..)),
            _ => false,
        }
    }

    pub fn not_found_unexpected(self) -> Self {
        match self {
            Self::NotFound(object, keys) => Self::UnexpectedNotFound(object, keys),
            _ => self,
        }
    }

    pub fn error_code(&self) -> String {
        match self {
            Self::NotFound(..) => "NotFound",
            Self::Duplicate(..) => "Duplicate",
            Self::InvalidFieldValue(..) => "InvalidFieldValue",
            Self::MissingRequiredField(..) => "MissingRequiredField",
            Self::ConcurrentUpdate(..) => "ConcurrentUpdate",
            Self::JsonError(json_err) => match json_err {
                JsonPayloadError::ContentType => "UnsupportedContentType",
                JsonPayloadError::Serialize(..) => "InternalServerError",
                JsonPayloadError::Deserialize(..) => "InvalidJson",
                JsonPayloadError::Overflow { .. } => "PayloadTooLarge",
                JsonPayloadError::OverflowKnownLength { .. } => "PayloadTooLarge",
                JsonPayloadError::Payload(..) => "InvalidPayload",
                _ => "UnknownPayloadError",
            },
            Self::QueryStringError(..) => "InvalidQueryString",
            Self::PathError(..) => "InvalidUrlPath",
            _ => "InternalServerError",
        }
        .into()
    }

    pub fn to_error_response(&self) -> ErrorResponse {
        let mut message = self.to_string();
        let mut keys: Option<Vec<FieldValue>> = None;
        let mut object: Option<ObjectKind> = None;
        let mut field: Option<FieldValue> = None;
        let mut allowed_values: Option<AllowedValues> = None;

        match self {
            Self::NotFound(o, k) => {
                object = Some(*o);
                keys = Some(k.0.clone());
            }
            Self::Duplicate(o, k) => {
                object = Some(*o);
                keys = Some(k.0.clone());
            }
            Self::InvalidFieldValue(fv, av) => {
                field = Some(fv.clone());
                allowed_values = Some(av.clone());
            }
            Self::MissingRequiredField(name, av) => {
                field = Some(FieldValue::null_value(name));
                allowed_values = Some(av.clone());
            }
            Self::ConcurrentUpdate(o, fv) => {
                object = Some(*o);
                keys = Some(fv.0.clone());
            }
            _ => {}
        }

        if self.is_internal_server_error() {
            message = "An Internal Server Error occurred.".into();
        }

        ErrorResponse {
            error_code: self.error_code(),
            message,
            object,
            keys,
            field,
            allowed_values,
        }
    }
}

impl ResponseError for TrackerError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(..) => StatusCode::NOT_FOUND,
            Self::Duplicate(..) => StatusCode::CONFLICT,
            Self::InvalidFieldValue(..) => StatusCode::BAD_REQUEST,
            Self::MissingRequiredField(..) => StatusCode::BAD_REQUEST,
            Self::ConcurrentUpdate(..) => StatusCode::CONFLICT,
            Self::SqlError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnexpectedNotFound(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JsonError(json_err) => match json_err {
                JsonPayloadError::ContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
                _ => json_err.status_code(),
            },
            Self::QueryStringError(..) => StatusCode::BAD_REQUEST,
            Self::PathError(..) => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.to_error_response())
    }
}
