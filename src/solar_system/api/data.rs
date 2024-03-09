use crate::{
    data::{PageRequest, PageRequestRaw},
    error::TrackerError,
    field::Field,
    field_names,
    game_save::api::SaveFields,
    solar_system::domain,
    utils::double_option,
};
use actix_web::{body::BoxBody, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarSystem {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub save_id: Uuid,
    pub name: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSolarSystemRequest {
    pub name: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSolarSystemRequest {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub notes: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequestRaw {
    #[serde(flatten)]
    pub page_request: PageRequestRaw,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub page_request: PageRequest<SolarSystemFields>,
    pub name: Option<String>,
}

impl From<domain::SolarSystem> for SolarSystem {
    fn from(value: domain::SolarSystem) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at,
            save_id: value.save_id,
            name: value.name,
            notes: value.notes,
        }
    }
}

impl Responder for SolarSystem {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        HttpResponse::Ok().json(self)
    }
}

impl TryFrom<SearchRequestRaw> for SearchRequest {
    type Error = TrackerError;

    fn try_from(value: SearchRequestRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            page_request: PageRequest::try_from(value.page_request)?,
            name: value.name,
        })
    }
}

field_names!(
    SolarSystemFields<domain::SolarSystemColumns> {
        Id => { value: "id", column: Id },
        Save(SaveFields) => { prefix: "save" },
        #[default]
        CreatedAt => { value: "created_at", column: CreatedAt },
        Name => { value: "name", column: Name },
        Notes => { value: "notes", column: Notes }
    }
);
