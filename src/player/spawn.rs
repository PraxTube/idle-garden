#[cfg(not(target_arch = "wasm32"))]
use std::fs::read_to_string;

use bevy::prelude::*;
use bevy_trickfilm::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::assets::PLAYER_SAVE_FILE;
#[cfg(target_arch = "wasm32")]
use crate::assets::WASM_PLAYER_KEY_STORAGE;
use crate::{
    world::{
        DynamicCollider, InitialFloraSpawned, StaticSensorCircle, Velocity, YSort,
        PLAYER_COLLISION_GROUPS,
    },
    GameAssets,
};

use super::Player;

pub const COLLIDER_RADIUS: f32 = 2.5;
pub const COLLIDER_OFFSET: Vec2 = Vec2::new(0.0, -12.5);
pub const DEFAULT_PLAYER_SPAWN_POS: Vec2 = Vec2::ZERO;

const SCYTHE_OFFSET: Vec3 = Vec3::new(-25.0, 0.0, 0.0);

#[derive(Component)]
pub struct Scythe {
    pub previous_dir: Vec2,
    pub delta_dir: f32,
}
#[derive(Component)]
pub struct ScytheGFX;

fn player_from_string(raw_player: &str) -> Vec2 {
    if raw_player.is_empty() {
        return DEFAULT_PLAYER_SPAWN_POS;
    }

    let parts = raw_player
        .split_once(',')
        .expect("failed to split player string in local storage");
    let x = parts.0.parse::<f32>().unwrap_or_default();
    let y = parts.1.parse::<f32>().unwrap_or_default();
    Vec2::new(x, y)
}

fn spawn_player_from_args(commands: &mut Commands, assets: &GameAssets, pos: Vec2) {
    let mut animator = AnimationPlayer2D::default();
    animator.play(assets.player_animations[0].clone()).repeat();

    let root = commands
        .spawn((
            Player::default(),
            PLAYER_COLLISION_GROUPS,
            Velocity::default(),
            DynamicCollider::new(COLLIDER_RADIUS, COLLIDER_OFFSET),
            StaticSensorCircle::new(COLLIDER_RADIUS, COLLIDER_OFFSET),
            animator,
            YSort(12.0),
            Sprite::from_atlas_image(assets.player.clone(), assets.player_layout.clone().into()),
            Transform::from_translation(pos.extend(0.0)),
        ))
        .id();

    let scythe_socket = commands
        .spawn((
            ChildOf(root),
            Scythe {
                previous_dir: Vec2::NEG_X,
                delta_dir: 1.0,
            },
            Transform::default(),
            Visibility::Inherited,
        ))
        .id();

    commands.spawn((
        ChildOf(scythe_socket),
        ScytheGFX,
        Sprite::from_image(assets.scythe.clone()),
        Transform::from_translation(SCYTHE_OFFSET),
    ));
}

#[cfg(target_arch = "wasm32")]
fn spawn_player_wasm(commands: &mut Commands, assets: &GameAssets) {
    use web_sys::window;

    let storage = window()
        .expect("failed to get window")
        .local_storage()
        .expect("failed to get local storage")
        .expect("failed to unwrap local storage");

    let raw_player = storage
        .get_item(WASM_PLAYER_KEY_STORAGE)
        .expect("failed to get local storage item WASM key")
        .unwrap_or_default();
    let pos = player_from_string(&raw_player);

    spawn_player_from_args(commands, assets, pos);
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_player_native(commands: &mut Commands, assets: &GameAssets) {
    let raw_player = read_to_string(PLAYER_SAVE_FILE).expect("failed to read player save file");
    let pos = player_from_string(&raw_player);
    spawn_player_from_args(commands, assets, pos);
}

fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
    #[cfg(target_arch = "wasm32")]
    spawn_player_wasm(&mut commands, &assets);

    #[cfg(not(target_arch = "wasm32"))]
    spawn_player_native(&mut commands, &assets);
}

pub struct PlayerSpawnPlugin;

impl Plugin for PlayerSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_player.run_if(
                resource_exists::<GameAssets>
                    .and(resource_exists::<InitialFloraSpawned>)
                    .and(run_once),
            ),
        );
    }
}
