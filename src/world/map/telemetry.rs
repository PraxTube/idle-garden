use bevy::{asset::uuid::Uuid, prelude::*};
use serde::{Deserialize, Serialize};

use super::ProgressionCore;

#[cfg(not(target_arch = "wasm32"))]
use crate::assets::GAME_TELEMETRY_FILE;
#[cfg(target_arch = "wasm32")]
use crate::assets::WASM_GAME_TELEMETRY_KEY_STORAGE;

/// Interval of sending data to server in seconds.
const DATA_UPLOAD_INTERVAL: usize = 60;

#[derive(Resource, Serialize, Deserialize)]
pub struct GameTelemetryManager {
    telemetries: Vec<GameTelemetry>,
    user_uuid: Uuid,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameTelemetry {
    cores: Vec<(u64, ProgressionCore)>,
    actions: Vec<(u64, usize)>,
}

impl GameTelemetryManager {
    fn empty() -> Self {
        Self {
            telemetries: Vec::new(),
            user_uuid: Uuid::new_v4(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn insert_game_telemetry_wasm(commands: &mut Commands) {
    use web_sys::window;

    let storage = window()
        .expect("failed to get window")
        .local_storage()
        .expect("failed to get local storage")
        .expect("failed to unwrap local storage");

    let telemetry = match storage
        .get_item(WASM_GAME_TELEMETRY_KEY_STORAGE)
        .expect("failed to get local storage item WASM key")
    {
        Some(r) => {
            serde_json::from_str(&r).expect("failed to parse progression core from json string")
        }
        None => GameTelemetryManager::empty(),
    };

    commands.insert_resource(telemetry);
}

#[cfg(not(target_arch = "wasm32"))]
fn insert_game_telemetry_native(commands: &mut Commands) {
    use std::fs::read_to_string;

    let raw_core =
        read_to_string(GAME_TELEMETRY_FILE).expect("failed to read progression core file");
    let core = if raw_core.is_empty() {
        GameTelemetryManager::empty()
    } else {
        serde_json::from_str(&raw_core).expect("failed to parse progression core from json string")
    };
    commands.insert_resource(core);
}

fn insert_game_telemetry_manager(mut commands: Commands) {
    #[cfg(not(target_arch = "wasm32"))]
    insert_game_telemetry_native(&mut commands);

    #[cfg(target_arch = "wasm32")]
    insert_game_telemetry_wasm(&mut commands);
}

pub struct GameTelemetryPlugin;

impl Plugin for GameTelemetryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, insert_game_telemetry_manager);
    }
}
