use uuid::Uuid;

pub enum ItemType {
    Component,
    Building
}

pub enum ItemSubType {
    CommonResource,
    RareResource,
    Miner,
    AssemblingMachine,
    ChemicalPlant,
    MatrixLab
}

pub struct Item {
    pub id: Uuid,
    pub created_at: u32,
    pub version: u32,
    pub name: String,
    pub item_type: ItemType,
    pub item_sub_type: Option<ItemSubType>,
    pub stack_size: u16,
    pub production_multiplier: Option<f32>,
    pub image_path: String
}

pub struct ItemRecipe {
    pub id: Uuid,
    pub created_at: u32,
    pub version: u32,
    pub name: String,
    pub craft_time_secs: f32
}

pub struct ItemRecipeInput {
    pub recipe_id: Uuid,
    pub item_id: Uuid,
    pub amount: u16,
    pub extra_products: bool,
    pub production_speedup: bool
}

pub struct ItemRecipeOutput {
    pub recipe_id: Uuid,
    pub item_id: Uuid,
    pub amount: u16
}