use bevy::prelude::*;
use bevy_trickfilm::prelude::*;

use crate::{
    world::{
        DynamicCollider, ProgressionCore, StaticSensorCircle, Velocity, YSort,
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
            YSort(12.0),
            Visibility::Inherited,
            Transform::from_translation(pos.extend(0.0)),
        ))
        .id();

    commands.spawn((
        ChildOf(root),
        Transform::from_scale(Vec3::splat(0.5)),
        animator,
        Sprite::from_atlas_image(assets.player.clone(), assets.player_layout.clone().into()),
    ));

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

fn spawn_player(mut commands: Commands, assets: Res<GameAssets>, core: Res<ProgressionCore>) {
    spawn_player_from_args(&mut commands, &assets, core.player);
}

pub struct PlayerSpawnPlugin;

impl Plugin for PlayerSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_player.run_if(
                resource_exists::<GameAssets>
                    .and(resource_exists::<ProgressionCore>)
                    .and(run_once),
            ),
        );
    }
}
