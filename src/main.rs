#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::needless_range_loop,
    clippy::field_reassign_with_default,
    clippy::approx_constant
)]

mod assets;
mod audio;
mod player;
mod ui;
mod world;

pub use assets::{EffectAssets, GameAssets};
pub type GameRng = rand_xoshiro::Xoshiro256PlusPlus;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowMode, WindowResolution};
use bevy_enoki::prelude::*;
use bevy_trickfilm::prelude::*;

use bevy_asset_loader::prelude::*;
#[cfg(debug_assertions)]
use world::simulate_progression;

const BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.62, 0.33);
const DEFAULT_WINDOW_WIDTH: f32 = 1280.0;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum GameState {
    #[default]
    AssetLoading,
    Gaming,
    Menu,
}

fn main() {
    #[cfg(debug_assertions)]
    use std::env::args;

    #[cfg(debug_assertions)]
    if args()
        .find(|s| s.to_lowercase() == "simulate-progression")
        .is_some()
    {
        simulate_progression();
        return;
    }

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Fifo,
                    mode: WindowMode::Windowed,
                    resolution: WindowResolution::new(
                        DEFAULT_WINDOW_WIDTH,
                        DEFAULT_WINDOW_WIDTH * 9.0 / 16.0,
                    ),
                    fit_canvas_to_parent: true,
                    #[cfg(all(not(debug_assertions), target_arch = "wasm32"))]
                    canvas: Some("#game-canvas".to_string()),
                    ..default()
                }),
                ..default()
            })
            .build(),
        Animation2DPlugin,
        EnokiPlugin,
    ))
    .init_state::<GameState>()
    .add_loading_state(
        LoadingState::new(GameState::AssetLoading)
            .continue_to_state(GameState::Gaming)
            .load_collection::<GameAssets>()
            .finally_init_resource::<EffectAssets>(),
    )
    .insert_resource(ClearColor(BACKGROUND_COLOR))
    .add_plugins((
        assets::GameAssetsPlugin,
        ui::UiPlugin,
        audio::GameAudioPlugin,
        world::WorldPlugin,
        player::PlayerPlugin,
    ));

    app.run();
}
