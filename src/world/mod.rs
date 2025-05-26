pub mod utils;

mod camera;
mod collisions;
mod debug;
mod map;

pub use camera::{MainCamera, YSort};
pub use collisions::{
    DynamicCollider, StaticSensorCircle, Velocity, PLAYER_COLLISION_GROUPS, SLASH_COLLISION_GROUPS,
};
pub use debug::DebugState;
pub use map::{Flora, ProgressionCore, ZLevel};

use bevy::prelude::*;

pub const TILE_SIZE: f32 = 16.0;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            debug::DebugPlugin,
            camera::CameraPlugin,
            collisions::WorldCollisionPlugin,
            map::MapPlugin,
        ));
    }
}
