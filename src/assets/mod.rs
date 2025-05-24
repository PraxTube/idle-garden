use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub const FLORA_DATA_CORE: &str = include_str!("../../assets/progression/flora.json");

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "player/player.png")]
    pub player: Handle<Image>,

    // --- MAP ---
    #[asset(path = "map/grass.png")]
    pub grass: Handle<Image>,

    #[asset(path = "map/potatoe.png")]
    pub potatoe: Handle<Image>,
    #[asset(path = "map/tree.png")]
    pub tree: Handle<Image>,
}
