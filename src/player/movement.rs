use bevy::prelude::*;
use bevy_trickfilm::prelude::*;

use crate::ui::{MenuAction, MenuActionEvent};
use crate::world::Velocity;
use crate::{GameAssets, GameState};

use super::input::GamingInput;
use super::spawn::DEFAULT_PLAYER_SPAWN_POS;
use super::{Player, MOVE_SPEED};

/// System set that updates the player velocity, but doesn't actually move the player.
/// That happens in the collision/physics system.
/// This only updates the "ideal" player velocity.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerMovementSystemSet;

fn reset_velocity(mut q_player: Query<&mut Velocity, With<Player>>) {
    let Ok(mut velocity) = q_player.single_mut() else {
        return;
    };
    *velocity = Velocity::default();
}

fn set_player_pos_to_default_on_reset(
    mut q_player: Query<&mut Transform, With<Player>>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    if !ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::Reset)
    {
        return;
    }

    let Ok(mut transform) = q_player.single_mut() else {
        return;
    };

    transform.translation = DEFAULT_PLAYER_SPAWN_POS.extend(0.0);
}

fn move_running(
    time: Res<Time>,
    gaming_input: Res<GamingInput>,
    mut q_player: Query<(&Player, &mut Velocity)>,
) {
    let Ok((_player, mut velocity)) = q_player.single_mut() else {
        return;
    };

    let direction = gaming_input.move_direction;
    velocity.0 = direction * MOVE_SPEED * time.delta_secs();
}

fn animate_player(
    assets: Res<GameAssets>,
    gaming_input: Res<GamingInput>,
    mut q_player: Query<(&mut Sprite, &mut AnimationPlayer2D), With<Player>>,
) {
    let Ok((mut sprite, mut animator)) = q_player.single_mut() else {
        return;
    };

    let animation = if gaming_input.move_direction == Vec2::ZERO {
        assets.player_animations[0].clone()
    } else {
        assets.player_animations[1].clone()
    };

    if gaming_input.move_direction.x > 0.0 {
        sprite.flip_x = false;
    } else if gaming_input.move_direction.x < 0.0 {
        sprite.flip_x = true;
    }

    animator.play(animation).repeat();
}

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (reset_velocity, set_player_pos_to_default_on_reset),
        )
        .add_systems(
            Update,
            (move_running, animate_player)
                .chain()
                .in_set(PlayerMovementSystemSet)
                .run_if(in_state(GameState::Gaming)),
        );
    }
}
