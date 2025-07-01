use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::{
    world::{
        utils::quat_from_vec2, Blueprint, BuildingSystemSet, ItemBought, ProgressionSystemSet,
        StaticSensorCircle, ZLevel, SLASH_COLLISION_GROUPS,
    },
    GameAssets, GameState,
};

use super::{GamingInput, Player};

const OFFSET_DIRECTION: Vec2 = Vec2::X;
const OFFSET_MAGNITUDE: f32 = 20.0;

#[derive(Component)]
struct Slash {
    timer: Timer,
}

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

fn spawn_slashes(
    mut commands: Commands,
    assets: Res<GameAssets>,
    gaming_input: Res<GamingInput>,
    q_blueprints: Query<&Blueprint>,
    q_player: Query<(&Transform, &Player)>,
) {
    if !gaming_input.slash {
        return;
    }
    if !q_blueprints.is_empty() {
        return;
    }

    let Ok((player_transform, player)) = q_player.single() else {
        return;
    };

    if player.is_over_ui {
        return;
    }

    let pos = player_transform.translation.xy()
        + OFFSET_DIRECTION.rotate(gaming_input.aim_direction) * OFFSET_MAGNITUDE;

    spawn_slash(
        &mut commands,
        &assets,
        pos,
        quat_from_vec2(gaming_input.aim_direction),
        true,
    );
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

pub struct PlayerSlashPlugin;

impl Plugin for PlayerSlashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                despawn_slashes,
                update_player_is_over_ui,
                spawn_slashes.run_if(resource_exists::<GameAssets>),
                spawn_slash_on_item_bought,
            )
                .chain()
                .run_if(in_state(GameState::Gaming))
                .after(ProgressionSystemSet)
                .before(BuildingSystemSet),
        );
    }
}
