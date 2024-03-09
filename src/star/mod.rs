pub mod api;
pub mod domain;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter};

#[derive(Debug, Copy, Clone, sqlx::Type, AsRefStr, EnumIter, Serialize, Deserialize)]
#[sqlx(type_name = "spectral_class", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SpectralClass {
    ClassA,
    ClassB,
    ClassF,
    ClassG,
    ClassK,
    ClassM,
    ClassO,
    RedGiant,
    YellowGiant,
    WhiteGiant,
    BlueGiant,
    WhiteDwarf,
    BlackHole,
    Neutron,
}

pub use api::config;
pub use domain::*;
