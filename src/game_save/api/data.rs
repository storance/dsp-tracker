use crate::field::Field;
use crate::{field_names, game_save::domain};
use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct CreateGameSaveRequest {
    pub name: String,
    pub notes: Option<String>,
    pub mining_speed: u32,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateGameSaveRequest {
    pub name: Option<String>,
    pub notes: Option<Option<String>>,
    pub mining_speed: Option<u32>,
}

#[derive(Deserialize, Serialize)]
pub struct GameSave {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub notes: Option<String>,
    pub mining_speed: u32,
}

impl Responder for GameSave {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok().json(self)
    }
}

impl From<domain::GameSave> for GameSave {
    fn from(value: domain::GameSave) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at,
            name: value.name,
            notes: None,
            mining_speed: value.mining_speed,
        }
    }
}

field_names!(
    SaveFields<domain::GameSaveColumns> {
        Id => { value: "id" },
        #[default]
        CreatedAt => { value: "created_at" },
        Name => { value: "name" },
        Notes => { value: "notes" }
    }
);
