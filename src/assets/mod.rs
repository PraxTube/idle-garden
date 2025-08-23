mod events;

pub use events::PlayerFootstepEvent;

use bevy::{asset::RenderAssetUsages, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_enoki::prelude::*;
use bevy_trickfilm::prelude::*;

use crate::world::GrassMaterial;

pub const APIKEY: &str = include_str!("../../apikey.env");

pub const FLORA_DATA_CORE: &str = include_str!("../../assets/progression/flora.json");
pub const FLORA_SHADER: &str = "shaders/flora_shader.wgsl";
pub const GRASS_SHADER: &str = "shaders/grass_shader.wgsl";
pub const CLOUDS_SHADER: &str = "shaders/clouds_shader.wgsl";

pub const HALF_HEIGHT_GRASS_TIMESTAMPS_IMAGE: u32 = 8;

#[cfg(not(target_arch = "wasm32"))]
pub const MAP_DATA_FILE: &str = "assets/save/map_data";
#[cfg(not(target_arch = "wasm32"))]
pub const PROGRESSION_CORE_FILE: &str = "assets/save/progression_core.json";
#[cfg(not(target_arch = "wasm32"))]
pub const GAME_TELEMETRY_FILE: &str = "assets/save/telemetry";
#[cfg(not(target_arch = "wasm32"))]
pub const CONSENT_FILE: &str = "assets/save/consent";

#[cfg(target_arch = "wasm32")]
pub const WASM_MAP_DATA_KEY_STORAGE: &str = "map-grid";
#[cfg(target_arch = "wasm32")]
pub const WASM_PROGRESSION_CORE_KEY_STORAGE: &str = "progression-core";
#[cfg(target_arch = "wasm32")]
pub const WASM_GAME_TELEMETRY_KEY_STORAGE: &str = "game-telemetry";
#[cfg(target_arch = "wasm32")]
pub const WASM_CONSENT_STORAGE: &str = "consent";
#[cfg(target_arch = "wasm32")]
pub const WASM_KEYS: [&str; 3] = [
    WASM_PROGRESSION_CORE_KEY_STORAGE,
    WASM_MAP_DATA_KEY_STORAGE,
    WASM_GAME_TELEMETRY_KEY_STORAGE,
];

const CUT_GRASS_PARTICLES_FILE: &str = "effects/cut_grass.ron";

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(events::GameAssetsEventsPlugin);
    }
}

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
    #[asset(path = "player/shadow.png")]
    pub player_shadow: Handle<Image>,

    #[asset(path = "player/scythe.png")]
    pub scythe: Handle<Image>,
    #[asset(path = "player/slash.png")]
    pub slash: Handle<Image>,

    // --- UI ---
    #[asset(
        paths(
            "ui/icons/potatoe_icon.png",
            "ui/icons/raddish_icon.png",
            "ui/icons/carrot_icon.png",
            "ui/icons/corn_icon.png",
            "ui/icons/pumpkin_icon.png",
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

    #[asset(path = "ui/discord.png")]
    pub discord_button: Handle<Image>,

    // --- MAP ---
    #[asset(path = "map/grass.png")]
    pub grass: Handle<Image>,
    #[asset(path = "map/grass_background.png")]
    pub grass_background_tile: Handle<Image>,

    #[asset(path = "map/building_selector.png")]
    pub building_selector: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 2, rows = 1))]
    pub building_selector_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "map/building_selector_animation.trickfilm.ron#main")]
    pub building_selector_animation: Handle<AnimationClip2D>,

    #[asset(
        paths(
            "map/potatoe.png",
            "map/raddish.png",
            "map/carrot.png",
            "map/corn.png",
            "map/pumpkin.png"
        ),
        collection(typed)
    )]
    pub flora_images: Vec<Handle<Image>>,
    #[asset(path = "map/corn_crop_left.png")]
    pub corn_crop_left: Handle<Image>,
    #[asset(path = "map/corn_crop_right.png")]
    pub corn_crop_right: Handle<Image>,

    #[asset(path = "map/fence_top_corner.png")]
    pub fence_top_corner: Handle<Image>,
    #[asset(path = "map/fence_bottom_corner.png")]
    pub fence_bottom_corner: Handle<Image>,
    #[asset(path = "map/fence_vertical.png")]
    pub fence_vertical: Handle<Image>,
    #[asset(path = "map/fence_horizontal.png")]
    pub fence_horizontal: Handle<Image>,

    // --- EFFECTS ---
    #[asset(path = "effects/grass_snippet.png")]
    pub grass_snippet: Handle<Image>,

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

    // --- AUDIO ---
    #[asset(
        paths(
            "audio/music/fireflies.ogg",
            "audio/music/new_life.ogg",
            "audio/music/frog_pond.ogg"
        ),
        collection(typed)
    )]
    pub bgms: Vec<Handle<AudioSource>>,
    #[asset(path = "audio/player_footstep.ogg")]
    pub player_footstep: Handle<AudioSource>,

    // --- FONTS ---
    #[asset(path = "fonts/PressStart2P.ttf")]
    pub pixel_font: Handle<Font>,
}

#[derive(Resource)]
pub struct EffectAssets {
    pub cut_grass_material: Handle<SpriteParticle2dMaterial>,
    pub cut_grass_particles: Handle<Particle2dEffect>,
    pub rect_mesh: Handle<Mesh>,
    pub grass_material: Handle<GrassMaterial>,
    pub grass_material_timestamps: Handle<Image>,
}

impl FromWorld for EffectAssets {
    fn from_world(world: &mut World) -> Self {
        let default_self_on_error = Self {
            cut_grass_material: Handle::<SpriteParticle2dMaterial>::default(),
            cut_grass_particles: Handle::<Particle2dEffect>::default(),
            rect_mesh: Handle::<Mesh>::default(),
            grass_material: Handle::<GrassMaterial>::default(),
            grass_material_timestamps: Handle::<Image>::default(),
        };

        let Some(mut meshes) = world.get_resource_mut::<Assets<Mesh>>() else {
            return default_self_on_error;
        };
        let rect_mesh = meshes.add(Rectangle::default());

        let testy_particles = world.load_asset(CUT_GRASS_PARTICLES_FILE);
        let Some(assets) = world.get_resource::<GameAssets>() else {
            error!("failed to get GameAssets, must be exist at this point");
            return default_self_on_error;
        };

        let handle = assets.grass_snippet.clone();

        let Some(mut particles) = world.get_resource_mut::<Assets<SpriteParticle2dMaterial>>()
        else {
            error!("failed to get Assets<SpriteParticle2dMaterial>, must be exist at this point");
            return default_self_on_error;
        };

        let default_particle_material =
            particles.add(SpriteParticle2dMaterial::from_texture(handle));

        let Some(mut images) = world.get_resource_mut::<Assets<Image>>() else {
            error!("failed to get Assets<Image>, must be exist at this point");
            return default_self_on_error;
        };

        let timestamps_image = Image::new_fill(
            bevy::render::render_resource::Extent3d {
                width: 4096,
                height: 2 * HALF_HEIGHT_GRASS_TIMESTAMPS_IMAGE,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            &f32::to_ne_bytes(-10.0),
            bevy::render::render_resource::TextureFormat::R32Float,
            RenderAssetUsages::default(),
        );

        let timestamps_handle = images.add(timestamps_image);

        let Some(assets) = world.get_resource::<GameAssets>() else {
            error!("failed to get GameAssets, must be exist at this point");
            return default_self_on_error;
        };

        let raw_grass_material = GrassMaterial {
            texture: Some(assets.grass.clone()),
            discrete_sine: Some(assets.discrete_sine_texture.clone()),
            discrete_exp_damp: Some(assets.discrete_exp_damp_texture.clone()),
            timestamps: Some(timestamps_handle.clone()),
        };

        let Some(mut grass_materials) = world.get_resource_mut::<Assets<GrassMaterial>>() else {
            error!("failed to get Assets<GrassMaterial>, must be exist at this point");
            return default_self_on_error;
        };

        let grass_material = grass_materials.add(raw_grass_material);

        Self {
            cut_grass_material: default_particle_material,
            cut_grass_particles: testy_particles,
            rect_mesh,
            grass_material,
            grass_material_timestamps: timestamps_handle,
        }
    }
}
