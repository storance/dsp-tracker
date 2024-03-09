use crate::star::SpectralClass;
use chrono::{DateTime, Utc};
use sea_query::Iden;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct Star {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub solar_system_id: Uuid,
    pub spectral_class: SpectralClass,
    pub luminosity: f32,
    pub radius: f32,
}

#[derive(Debug, Copy, Clone, Iden)]
#[allow(dead_code)]
pub enum StarColumns {
    #[iden(rename = "stars")]
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    Version,
    SolarSystemId,
    SpectralClass,
    Luminosity,
    Radius,
}

impl Star {
    pub fn new(
        solar_system_id: Uuid,
        spectral_class: SpectralClass,
        luminosity: f32,
        radius: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: None,
            version: 0,
            solar_system_id,
            spectral_class,
            luminosity,
            radius,
        }
    }
}

impl From<StarColumns> for String {
    fn from(value: StarColumns) -> Self {
        value.to_string()
    }
}
