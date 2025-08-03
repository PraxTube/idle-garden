use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::{
    world::{
        utils::quat_from_vec2, BuildingSystemSet, ItemBought, ProgressionSystemSet,
        StaticSensorCircle, ZLevel, SLASH_COLLISION_GROUPS,
    },
    GameAssets, GameState,
};

use super::{
    spawn::{Scythe, ScytheGFX},
    GamingInput, Player,
};

#[derive(Component)]
struct Slash {
    timer: Timer,
}

/// Event used to signal that a slash was spawned.
/// Necessary for game telemetry, can probably be remove after Bachelor.
#[derive(Event)]
pub struct SpawnedSlash;

impl Default for Slash {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.15, TimerMode::Once),
        }
    }
}

fn spawn_slash(
    commands: &mut Commands,
    assets: &GameAssets,
    pos: Vec2,
    rotation: Quat,
    visible: bool,
) {
    let visibility = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    commands.spawn((
        Slash::default(),
        Transform::from_translation(pos.extend(ZLevel::TopEnvironment.value()))
            .with_rotation(rotation),
        Sprite::from_image(assets.slash.clone()),
        StaticSensorCircle::new(8.0, Vec2::ZERO),
        SLASH_COLLISION_GROUPS,
        visibility,
    ));
}

fn spawn_slash_on_item_bought(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_item_bought: EventReader<ItemBought>,
) {
    for ev in ev_item_bought.read() {
        spawn_slash(&mut commands, &assets, ev.pos, Quat::IDENTITY, false);
    }
}

fn despawn_slashes(
    mut commands: Commands,
    time: Res<Time>,
    mut q_slashes: Query<(Entity, &mut Slash)>,
) {
    for (entity, mut slash) in &mut q_slashes {
        slash.timer.tick(time.delta());
        if slash.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn update_player_is_over_ui(
    mut q_player: Query<&mut Player>,
    q_relative_cursor_positions: Query<&RelativeCursorPosition>,
) {
    let Ok(mut player) = q_player.single_mut() else {
        return;
    };

    player.is_over_ui = false;
    for cursor_pos in &q_relative_cursor_positions {
        if cursor_pos.mouse_over() {
            player.is_over_ui = true;
            break;
        }
    }
}

fn update_scythe_rotation(
    gaming_input: Res<GamingInput>,
    q_player: Single<&Transform, With<Player>>,
    q_scythe: Single<(&mut Transform, &mut Scythe), Without<Player>>,
) {
    let player_transform = q_player.into_inner();
    let (mut scythe_transform, mut scythe) = q_scythe.into_inner();

    let dir =
        (gaming_input.mouse_world_coords - player_transform.translation.xy()).normalize_or_zero();

    scythe.delta_dir = if dir.x * scythe.previous_dir.y - dir.y * scythe.previous_dir.x <= 0.0 {
        0.0
    } else {
        scythe.previous_dir.dot(dir)
    };

    scythe.previous_dir = dir;

    scythe_transform.rotation = quat_from_vec2(dir);
}

fn spawn_slashes_on_scythe_move(
    mut commands: Commands,
    assets: Res<GameAssets>,
    q_scythe: Single<&Scythe>,
    q_scythe_gfx: Single<&GlobalTransform, With<ScytheGFX>>,
) {
    let scythe = q_scythe.into_inner();
    let scythe_gfx_transform = q_scythe_gfx.into_inner();

    let pos = scythe_gfx_transform.translation().xy();

    if scythe.delta_dir > 0.8 && scythe.delta_dir < 0.98 {
        spawn_slash(&mut commands, &assets, pos, Quat::IDENTITY, false);
    }
}

pub struct PlayerSlashPlugin;

impl Plugin for PlayerSlashPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnedSlash>().add_systems(
            Update,
            (
                despawn_slashes,
                update_player_is_over_ui,
                update_scythe_rotation,
                spawn_slashes_on_scythe_move,
                spawn_slash_on_item_bought,
            )
                .chain()
                .run_if(in_state(GameState::Gaming))
                .after(ProgressionSystemSet)
                .before(BuildingSystemSet),
        );
    }
}
