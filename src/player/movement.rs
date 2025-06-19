use bevy::prelude::*;

use crate::ui::{MenuAction, MenuActionEvent};
use crate::world::Velocity;
use crate::GameState;

use super::input::GamingInput;
use super::spawn::DEFAULT_PLAYER_SPAWN_POS;
use super::{Player, MOVE_SPEED};

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

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (reset_velocity, set_player_pos_to_default_on_reset),
        )
        .add_systems(Update, move_running.run_if(in_state(GameState::Gaming)));
    }
}
