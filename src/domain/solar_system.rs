use uuid::Uuid;

pub enum SpectralClass {
    A,
    B,
    F,
    G,
    K,
    M,
    O,
    RedGiant,
    YellowGiant,
    WhiteGiant,
    BlueGiant,
    WhiteDwarf,
    BlackHole,
    Neutron
}

pub struct SolarSystem {
    pub id: Uuid,
    pub created_at: u32,
    pub version: u32,
    pub name: String,
    pub save_id: Uuid,
    pub star_spectral_class: SpectralClass,
    pub star_luminosity: f32,
    pub star_radius: f32
}

pub enum OceanType {
    Water,
    Lava,
    Ice,
    SulfuricAcid
}

pub struct PlanetType {
    pub id: Uuid,
    pub created_at: u32,
    pub version: u32,
    pub name: String,
    pub ocean_type: Option<OceanType>,
    pub wind_energy_percent: u16,
}

pub enum RotationDirection {
    Normal,
    Reverse,
    Horizontal
}

pub struct Planet {
    pub id: Uuid,
    pub created_at: u32,
    pub version: u32,
    pub name: String,
    pub solar_system_id: Uuid,
    pub parent_planet_id: Option<Uuid>,
    pub planet_type_id: Uuid,
    pub orbital_resonance: Option<f32>,
    pub rotation_direction: RotationDirection,
    pub solar_energy_percent: u16,
}

pub struct PlanetTypeAllowedResource {
    pub planet_type_id: Uuid,
    pub item_id: Uuid,
    pub star_spectral_class: Option<SpectralClass>,
}

pub struct PlanetAvailableResource {
    pub planet_id: Uuid,
    pub item_id: Uuid
}
