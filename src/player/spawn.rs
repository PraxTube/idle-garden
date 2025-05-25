use bevy::prelude::*;

use crate::{
    world::{DynamicCollider, Velocity, YSort, PLAYER_COLLISION_GROUPS},
    GameAssets, GameState,
};

use super::Player;

pub const COLLIDER_RADIUS: f32 = 2.5;
pub const COLLIDER_OFFSET: Vec2 = Vec2::new(0.0, -8.0);

fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        Player,
        PLAYER_COLLISION_GROUPS,
        Velocity::default(),
        DynamicCollider::new(COLLIDER_RADIUS, COLLIDER_OFFSET),
        YSort(5.0),
        Sprite::from_image(assets.player.clone()),
    ));
}

pub struct PlayerSpawnPlugin;

impl Plugin for PlayerSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_player);
    }
}
