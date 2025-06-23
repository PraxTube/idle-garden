use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub const FLORA_DATA_CORE: &str = include_str!("../../assets/progression/flora.json");
pub const FLORA_SHADER: &str = "shaders/flora_shader.wgsl";
pub const GRASS_SHADER: &str = "shaders/grass_shader.wgsl";

#[cfg(not(target_arch = "wasm32"))]
pub const MAP_DATA_FILE: &str = "assets/save/map_data";
#[cfg(not(target_arch = "wasm32"))]
pub const PROGRESSION_CORE_FILE: &str = "assets/save/progression_core.json";
#[cfg(not(target_arch = "wasm32"))]
pub const PLAYER_SAVE_FILE: &str = "assets/save/player";

#[cfg(target_arch = "wasm32")]
pub const WASM_MAP_DATA_KEY_STORAGE: &str = "map-grid";
#[cfg(target_arch = "wasm32")]
pub const WASM_PROGRESSION_CORE_KEY_STORAGE: &str = "progression-core";
#[cfg(target_arch = "wasm32")]
pub const WASM_PLAYER_KEY_STORAGE: &str = "player";
#[cfg(target_arch = "wasm32")]
pub const WASM_KEYS: [&str; 3] = [
    WASM_PROGRESSION_CORE_KEY_STORAGE,
    WASM_MAP_DATA_KEY_STORAGE,
    WASM_PLAYER_KEY_STORAGE,
];

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

    #[asset(path = "ui/store_bar.png")]
    pub store_bar: Handle<Image>,
    #[asset(path = "ui/store_item_background.png")]
    pub store_item_background: Handle<Image>,
    #[asset(path = "ui/store_item_unaffordable_overlay.png")]
    pub store_item_unaffordable_overlay: Handle<Image>,

    #[asset(path = "ui/menu_background.png")]
    pub menu_background: Handle<Image>,
    #[asset(path = "ui/reset_pop_up_background.png")]
    pub reset_pop_up_background: Handle<Image>,

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

    #[asset(path = "map/fence_top_corner.png")]
    pub fence_top_corner: Handle<Image>,
    #[asset(path = "map/fence_bottom_corner.png")]
    pub fence_bottom_corner: Handle<Image>,
    #[asset(path = "map/fence_vertical.png")]
    pub fence_vertical: Handle<Image>,
    #[asset(path = "map/fence_horizontal.png")]
    pub fence_horizontal: Handle<Image>,

    // --- SHADERS ---
    #[asset(path = "shaders/noise_texture.png")]
    pub noise_texture: Handle<Image>,
    #[asset(path = "shaders/discrete_sine.png")]
    pub discrete_sine_texture: Handle<Image>,
    #[asset(path = "shaders/discrete_exp_damp.png")]
    pub discrete_exp_damp_texture: Handle<Image>,

    // --- FONTS ---
    #[asset(path = "fonts/PressStart2P.ttf")]
    pub pixel_font: Handle<Font>,
}
