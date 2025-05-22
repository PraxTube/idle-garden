use bevy::prelude::*;

use crate::world::Velocity;
use crate::GameState;

use super::input::GamingInput;
use super::{Player, MOVE_SPEED};

fn reset_velocity(mut q_player: Query<&mut Velocity, With<Player>>) {
    let Ok(mut velocity) = q_player.single_mut() else {
        return;
    };
    *velocity = Velocity::default();
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
        app.add_systems(PreUpdate, reset_velocity)
            .add_systems(Update, move_running.run_if(in_state(GameState::Gaming)));
    }
}
