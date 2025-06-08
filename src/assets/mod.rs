use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub const FLORA_DATA_CORE: &str = include_str!("../../assets/progression/flora.json");
#[cfg(target_arch = "wasm32")]
pub const WASM_MAP_DATA_KEY_STORAGE: &str = "map-grid";
#[cfg(target_arch = "wasm32")]
pub const WASM_PROGRESSION_CORE_KEY_STORAGE: &str = "progression-core";
pub const FLORA_SHADER: &str = "shaders/flora_shader.wgsl";

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "player/player.png")]
    pub player: Handle<Image>,
    #[asset(path = "player/slash.png")]
    pub slash: Handle<Image>,

    // --- UI ---
    #[asset(
        paths(
            "ui/icons/potatoe_icon.png",
            "ui/icons/raddish_icon.png",
            "ui/icons/carrot_icon.png",
            "ui/icons/sunflower_icon.png",
            "map/tree.png",
            "map/swamp_tree.png"
        ),
        collection(typed)
    )]
    pub flora_icons: Vec<Handle<Image>>,

    #[asset(path = "ui/building_grid.png")]
    pub building_grid: Handle<Image>,

    // --- MAP ---
    #[asset(path = "map/grass.png")]
    pub grass: Handle<Image>,

    #[asset(
        paths(
            "map/potatoe.png",
            "map/raddish.png",
            "map/carrot.png",
            "map/sunflower.png",
            "map/tree.png",
            "map/swamp_tree.png",
        ),
        collection(typed)
    )]
    pub flora_images: Vec<Handle<Image>>,

    #[asset(path = "map/pine-tree.png")]
    pub pine_tree: Handle<Image>,

    // --- SHADERS ---
    #[asset(path = "shaders/noise_texture.png")]
    pub noise_texture: Handle<Image>,

    // --- FONTS ---
    #[asset(path = "fonts/PressStart2P.ttf")]
    pub pixel_font: Handle<Font>,
}
