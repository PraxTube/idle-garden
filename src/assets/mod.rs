use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_trickfilm::prelude::*;

pub const APIKEY: &str = include_str!("../../apikey.env");

pub const FLORA_DATA_CORE: &str = include_str!("../../assets/progression/flora.json");
pub const FLORA_SHADER: &str = "shaders/flora_shader.wgsl";
pub const GRASS_SHADER: &str = "shaders/grass_shader.wgsl";
pub const CLOUDS_SHADER: &str = "shaders/clouds_shader.wgsl";

#[cfg(not(target_arch = "wasm32"))]
pub const MAP_DATA_FILE: &str = "assets/save/map_data";
#[cfg(not(target_arch = "wasm32"))]
pub const PROGRESSION_CORE_FILE: &str = "assets/save/progression_core.json";
#[cfg(not(target_arch = "wasm32"))]
pub const PLAYER_SAVE_FILE: &str = "assets/save/player";
#[cfg(not(target_arch = "wasm32"))]
pub const GAME_TELEMETRY_FILE: &str = "assets/save/telemetry";
#[cfg(not(target_arch = "wasm32"))]
pub const CONSENT_FILE: &str = "assets/save/consent";

#[cfg(target_arch = "wasm32")]
pub const WASM_MAP_DATA_KEY_STORAGE: &str = "map-grid";
#[cfg(target_arch = "wasm32")]
pub const WASM_PROGRESSION_CORE_KEY_STORAGE: &str = "progression-core";
#[cfg(target_arch = "wasm32")]
pub const WASM_PLAYER_KEY_STORAGE: &str = "game-telemetry";
#[cfg(target_arch = "wasm32")]
pub const WASM_GAME_TELEMETRY_KEY_STORAGE: &str = "game-telemetry";
#[cfg(target_arch = "wasm32")]
pub const WASM_CONSENT_STORAGE: &str = "consent";
#[cfg(target_arch = "wasm32")]
pub const WASM_KEYS: [&str; 4] = [
    WASM_PROGRESSION_CORE_KEY_STORAGE,
    WASM_MAP_DATA_KEY_STORAGE,
    WASM_PLAYER_KEY_STORAGE,
    WASM_GAME_TELEMETRY_KEY_STORAGE,
];

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "player/player.png")]
    pub player: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 48, tile_size_y = 32, columns = 12, rows = 2))]
    pub player_layout: Handle<TextureAtlasLayout>,
    #[asset(
        paths(
            "player/animations.trickfilm.ron#idle",
            "player/animations.trickfilm.ron#run",
        ),
        collection(typed)
    )]
    pub player_animations: Vec<Handle<AnimationClip2D>>,

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
            "map/swamp_tree.png",
            "map/silo.png",
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
            "map/silo.png",
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
    #[asset(path = "shaders/discrete_sine.png")]
    pub discrete_sine_texture: Handle<Image>,
    #[asset(path = "shaders/discrete_exp_damp.png")]
    pub discrete_exp_damp_texture: Handle<Image>,
    #[asset(path = "shaders/primary_clouds_noise.png")]
    pub primary_clouds_noise_texture: Handle<Image>,
    #[asset(path = "shaders/secondary_clouds_noise.png")]
    pub secondary_clouds_noise_texture: Handle<Image>,
    #[asset(path = "shaders/tertiary_clouds_noise.png")]
    pub tertiary_clouds_noise_texture: Handle<Image>,
    #[asset(path = "shaders/quaternary_clouds_noise.png")]
    pub quaternary_clouds_noise_texture: Handle<Image>,

    // --- FONTS ---
    #[asset(path = "fonts/PressStart2P.ttf")]
    pub pixel_font: Handle<Font>,
}
