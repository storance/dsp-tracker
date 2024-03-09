#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "ocean_type")]
pub enum OceanType {
    Water,
    Lava,
    Ice,
    SulfuricAcid,
}
// CREATE TYPE ocean_type AS ENUM ('Lava', 'Ice', 'SulfuricAcid', 'Water');
