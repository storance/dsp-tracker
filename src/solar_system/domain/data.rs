use chrono::{DateTime, Utc};
use sea_query::Iden;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct SolarSystem {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub save_id: Uuid,
    pub name: String,
    pub notes: Option<String>,
}

#[derive(Debug, Copy, Clone, Iden)]
#[allow(dead_code)]
pub enum SolarSystemColumns {
    #[iden(rename = "solar_systems")]
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    Version,
    SaveId,
    Name,
    Notes,
}

impl SolarSystem {
    pub fn new(save_id: Uuid, name: String, notes: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: None,
            version: 0,
            save_id,
            name,
            notes,
        }
    }
}

impl From<SolarSystemColumns> for String {
    fn from(value: SolarSystemColumns) -> Self {
        value.to_string()
    }
}
