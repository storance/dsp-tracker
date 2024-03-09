use crate::field::Field;
use crate::field_names;
use crate::solar_system::api::SolarSystemFields;
use crate::star::SpectralClass;
use crate::star::{domain, StarColumns};
use actix_web::{body::BoxBody, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Star {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub solar_system_id: Uuid,
    pub spectral_class: SpectralClass,
    pub luminosity: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStarRequest {
    pub spectral_class: SpectralClass,
    pub luminosity: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStarRequest {
    pub spectral_class: Option<SpectralClass>,
    pub luminosity: Option<f32>,
    pub radius: Option<f32>,
}

impl From<domain::Star> for Star {
    fn from(value: domain::Star) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at,
            solar_system_id: value.solar_system_id,
            spectral_class: value.spectral_class,
            luminosity: value.luminosity,
            radius: value.radius,
        }
    }
}

impl Responder for Star {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        HttpResponse::Ok().json(self)
    }
}

field_names!(
    StarFields<StarColumns> {
        Id => { value: "id" },
        SolarSystem(SolarSystemFields) => { prefix: "solar_system" },
        #[default]
        CreatedAt => { value: "created_at" },
        SpectralClass => { value: "notes" },
        Luminosity => { value: "luminosity" },
        Radius => { value: "radius" },
    }
);
