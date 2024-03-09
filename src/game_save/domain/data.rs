use chrono::{DateTime, Utc};
use sea_query::Iden;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct GameSave {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub version: i32,
    pub name: String,
    pub notes: Option<String>,
    #[sqlx(try_from = "i32")]
    pub mining_speed: u32,
}

#[derive(Debug, Copy, Clone, Iden)]
#[allow(dead_code)]
pub enum GameSaveColumns {
    #[iden(rename = "saves")]
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    Version,
    Name,
    Notes,
    MiningSpeed,
}

impl From<GameSaveColumns> for String {
    fn from(value: GameSaveColumns) -> Self {
        value.to_string()
    }
}

impl GameSave {
    pub fn new(name: String, notes: Option<String>, mining_speed: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: None,
            version: 0,
            name,
            notes,
            mining_speed,
        }
    }
}
