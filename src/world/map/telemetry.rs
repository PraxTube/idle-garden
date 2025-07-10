use std::time::Duration;

use serde::{Deserialize, Serialize};

use bevy::{asset::uuid::Uuid, prelude::*, time::common_conditions::on_real_timer};
use bevy_mod_reqwest::*;

use super::{timestamp, ProgressionCore};

use crate::assets::APIKEY;
#[cfg(not(target_arch = "wasm32"))]
use crate::assets::GAME_TELEMETRY_FILE;
#[cfg(target_arch = "wasm32")]
use crate::assets::WASM_GAME_TELEMETRY_KEY_STORAGE;

/// Interval of sending data to server in seconds.
const DATA_UPLOAD_INTERVAL: u64 = 60;

const POST_URL: &str = "https://rancic.org:/telemetry";

#[derive(Component)]
struct PostRequestMarker;

#[derive(Resource, Serialize, Deserialize)]
pub struct GameTelemetryManager {
    telemetries: Vec<GameTelemetry>,
    user_uuid: Uuid,
    non_ok_responses: Vec<String>,
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
            non_ok_responses: Vec::new(),
        }
    }

    /// Clear the telemetries and responses, of course don't clear the UUID.
    fn clear(&mut self) {
        self.telemetries = Vec::new();
        self.non_ok_responses = Vec::new();
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

fn send_data_to_server(telemetry: Res<GameTelemetryManager>, mut client: BevyReqwest) {
    let payload = serde_json::to_string(&*telemetry)
        .expect("failed to parse GameTelemetryManager to json string");
    let hmac = generate_hmac(&payload);
    let url = format!("{}/{}", POST_URL, hmac);

    let req = client
        .post(url)
        .body(payload)
        .build()
        .expect("failed to build reqwest request");

    client
        .send(req)
        .on_response(
            |trigger: Trigger<ReqwestResponseEvent>,
             mut telemetry: ResMut<GameTelemetryManager>| {
                if trigger.event().status() == StatusCode::OK {
                    telemetry.clear();
                } else {
                    let msg = format!(
                        "[{}]: response status code was not 200: {}",
                        timestamp(),
                        trigger.event().status()
                    );
                    telemetry.non_ok_responses.push(msg);
                }
            },
        )
        .on_error(
            |trigger: Trigger<ReqwestErrorEvent>, mut telemetry: ResMut<GameTelemetryManager>| {
                let msg = format!("[{}]: {}", timestamp(), trigger.event().0);
                telemetry.non_ok_responses.push(msg);
            },
        );
}

fn generate_hmac(payload: &str) -> String {
    use hex::encode;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let mut mac =
        HmacSha256::new_from_slice(APIKEY.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());

    let result = mac.finalize();
    let code_bytes = result.into_bytes();

    encode(code_bytes)
}

pub struct GameTelemetryPlugin;

impl Plugin for GameTelemetryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ReqwestPlugin::default())
            .add_systems(Startup, insert_game_telemetry_manager)
            .add_systems(
                Update,
                send_data_to_server.run_if(
                    resource_exists::<GameTelemetryManager>
                        .and(on_real_timer(Duration::from_secs(DATA_UPLOAD_INTERVAL))),
                ),
            );
    }
}
