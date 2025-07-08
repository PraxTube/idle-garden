#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::needless_range_loop,
    clippy::field_reassign_with_default,
    clippy::approx_constant
)]

mod assets;
mod player;
mod ui;
mod world;

pub use assets::GameAssets;
pub type GameRng = rand_xoshiro::Xoshiro256PlusPlus;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowMode, WindowResolution};

use bevy_asset_loader::prelude::*;
#[cfg(debug_assertions)]
use world::simulate_progression;

const BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.62, 0.33);
const DEFAULT_WINDOW_WIDTH: f32 = 1280.0;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum GameState {
    #[default]
    AssetLoading,
    BachelorToggle,
    Gaming,
    Menu,
}

#[derive(Resource)]
pub struct BachelorBuild {
    with_building: bool,
}
#[derive(Component)]
struct BachelorBuildComponent;

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

    App::new()
        .add_systems(Startup, spawn_bachelor_toggle)
        .add_systems(
            Update,
            continue_from_bachelor_state.run_if(in_state(GameState::BachelorToggle)),
        )
        .add_plugins((DefaultPlugins
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
            .build(),))
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::BachelorToggle)
                .load_collection::<GameAssets>(),
        )
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins((ui::UiPlugin, world::WorldPlugin, player::PlayerPlugin))
        .run();
}

fn spawn_bachelor_toggle(mut commands: Commands) {
    commands.spawn((
        BachelorBuildComponent,
        GlobalZIndex(1),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ImageNode {
            image: Handle::<Image>::default(),
            color: Color::BLACK,
            ..default()
        },
    ));

    commands.spawn((
        BachelorBuildComponent,
        GlobalZIndex(2),
        Text::new("Select Build\nPress [1] to select WITHOUT building\nPress [2] to select WITH building."),
        TextFont {
            font_size: 35.0,
            font_smoothing: bevy::text::FontSmoothing::None,
            ..default()
        },
    ));
}

fn continue_from_bachelor_state(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
    q_bachelor_components: Query<Entity, With<BachelorBuildComponent>>,
) {
    if !keys.just_pressed(KeyCode::Digit1) && !keys.just_pressed(KeyCode::Digit2) {
        return;
    }

    let with_building = !keys.just_pressed(KeyCode::Digit1) | keys.just_pressed(KeyCode::Digit2);
    commands.insert_resource(BachelorBuild { with_building });
    next_state.set(GameState::Gaming);

    for entity in &q_bachelor_components {
        commands.entity(entity).despawn();
    }
}
