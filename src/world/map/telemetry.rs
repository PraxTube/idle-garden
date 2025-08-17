use std::time::Duration;

use serde::{Deserialize, Serialize};

use bevy::{asset::uuid::Uuid, prelude::*, time::common_conditions::on_real_timer};
use bevy_mod_reqwest::*;

use super::{timestamp, ItemBought, ProgressionCore};

#[cfg(not(target_arch = "wasm32"))]
use crate::assets::GAME_TELEMETRY_FILE;
#[cfg(target_arch = "wasm32")]
use crate::assets::WASM_GAME_TELEMETRY_KEY_STORAGE;
use crate::{
    assets::APIKEY,
    player::{Player, PlayerMovementSystemSet, SpawnedSlash},
    ui::Consent,
    world::Velocity,
};

/// Interval of sending data to server in seconds.
const DATA_UPLOAD_INTERVAL: u64 = 60;
const PROGRESSION_CORE_INTERVAL: u64 = 1;

const POST_URL: &str = "https://rancic.org:/telemetry";

#[derive(Component)]
struct PostRequestMarker;

#[derive(Resource, Serialize, Deserialize)]
pub struct GameTelemetryManager {
    telemetries: Vec<GameTelemetry>,
    pub id: Uuid,
    responses: Vec<String>,
}

/// Game telemetry of one interval (around 60 probably).
/// Cores get added about once a second. Actions get added on demand.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct GameTelemetry {
    // We don't need a timestamp here because we have one implicitely.
    // The `previous_timestamp` on `ProgressionCore` has the exact same timestamp
    // anyways, we can just use that.
    cores: Vec<ProgressionCore>,
    actions: Vec<(u64, usize)>,
}

enum TelemetryActions {
    StartedMoving,
    StoppedMoving,
    Slash,
    ItemBought,
}

impl Default for GameTelemetryManager {
    fn default() -> Self {
        Self {
            telemetries: Vec::new(),
            id: Uuid::new_v4(),
            responses: Vec::new(),
        }
    }
}

impl GameTelemetryManager {
    /// Clear the telemetries and responses, of course don't clear the UUID.
    fn clear(&mut self) {
        self.responses.clear();

        if self.telemetries.len() > 1 {
            self.telemetries.drain(0..self.telemetries.len() - 1);
        } else {
            error!(
                "game telemetry should contain at least 2 GameTelemetry entries, number of entries: {}",
                self.telemetries.len()
            );
        }
    }

    fn last_index(&mut self) -> usize {
        if self.telemetries.is_empty() {
            self.telemetries.push(GameTelemetry::default());
        }

        self.telemetries.len() - 1
    }
}

impl TelemetryActions {
    fn index(self) -> usize {
        match self {
            TelemetryActions::StartedMoving => 0,
            TelemetryActions::StoppedMoving => 1,
            TelemetryActions::Slash => 2,
            TelemetryActions::ItemBought => 3,
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn insert_game_telemetry_wasm(commands: &mut Commands) {
    use web_sys::window;

    let storage = window().and_then(|w| w.local_storage().ok()).flatten();

    let telemetry: GameTelemetryManager = storage
        .as_ref()
        .and_then(|s| s.get_item(WASM_GAME_TELEMETRY_KEY_STORAGE).ok().flatten())
        .and_then(|r| serde_json::from_str(&r).ok())
        .unwrap_or_default();

    commands.insert_resource(telemetry);
}

#[cfg(not(target_arch = "wasm32"))]
fn insert_game_telemetry_native(commands: &mut Commands) {
    use std::fs::read_to_string;

    let raw_core =
        read_to_string(GAME_TELEMETRY_FILE).expect("failed to read progression core file");
    let core = if raw_core.is_empty() {
        GameTelemetryManager::default()
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

fn send_data_to_server(
    telemetry: Res<GameTelemetryManager>,
    consent: Res<Consent>,
    mut client: BevyReqwest,
) {
    if !consent.0 {
        return;
    }

    let payload = serde_json::to_string(&*telemetry)
        .unwrap_or_else(|_| "FAILED TO SERIALIZE GAME TELEMETRY, WOOPS".to_string());
    let hmac = generate_hmac(&payload);
    let url = format!("{}/{}", POST_URL, hmac);

    let Ok(req) = client.post(url).body(payload).build() else {
        error!("failed to build Request (for telemetry)");
        return;
    };

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
                    telemetry.responses.push(msg);
                }
            },
        )
        .on_error(
            |trigger: Trigger<ReqwestErrorEvent>, mut telemetry: ResMut<GameTelemetryManager>| {
                let msg = format!("[{}]: {}", timestamp(), trigger.event().0);
                telemetry.responses.push(msg);
            },
        );
}

fn generate_hmac(payload: &str) -> String {
    use hex::encode;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let Ok(mut mac) = HmacSha256::new_from_slice(APIKEY.as_bytes()) else {
        error!("failed to generate Hmac from slice (API KEY), must never happen!");
        return String::default();
    };
    mac.update(payload.as_bytes());

    let result = mac.finalize();
    let code_bytes = result.into_bytes();

    encode(code_bytes)
}

fn insert_new_game_telemetry(mut telemetry: ResMut<GameTelemetryManager>, consent: Res<Consent>) {
    if !consent.0 {
        return;
    }
    telemetry.telemetries.push(GameTelemetry::default());
}

fn add_progression_core_to_telemetry_manager(
    core: Res<ProgressionCore>,
    consent: Res<Consent>,
    mut telemetry: ResMut<GameTelemetryManager>,
) {
    if !consent.0 {
        return;
    }

    if telemetry.telemetries.is_empty() {
        telemetry.telemetries.push(GameTelemetry::default());
    }

    let index = telemetry.telemetries.len() - 1;
    telemetry.telemetries[index].cores.push(core.clone());
}

fn add_telemetry_actions(
    mut telemetry: ResMut<GameTelemetryManager>,
    consent: Res<Consent>,
    q_player: Query<&Velocity, With<Player>>,
    mut ev_item_bought: EventReader<ItemBought>,
    mut ev_spawned_slash: EventReader<SpawnedSlash>,
    mut player_was_moving: Local<bool>,
) {
    if !consent.0 {
        return;
    }

    let index = telemetry.last_index();
    let timestamp = timestamp();

    for _ in ev_item_bought.read() {
        telemetry.telemetries[index]
            .actions
            .push((timestamp, TelemetryActions::ItemBought.index()));
    }

    for _ in ev_spawned_slash.read() {
        telemetry.telemetries[index]
            .actions
            .push((timestamp, TelemetryActions::Slash.index()));
    }

    let Ok(player_velocity) = q_player.single() else {
        return;
    };

    if player_velocity.0 == Vec2::ZERO && *player_was_moving {
        telemetry.telemetries[index]
            .actions
            .push((timestamp, TelemetryActions::StoppedMoving.index()));
    } else if player_velocity.0 != Vec2::ZERO && !*player_was_moving {
        telemetry.telemetries[index]
            .actions
            .push((timestamp, TelemetryActions::StartedMoving.index()));
    }

    *player_was_moving = player_velocity.0 != Vec2::ZERO;
}

pub struct GameTelemetryPlugin;

impl Plugin for GameTelemetryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ReqwestPlugin::default())
            .add_systems(Startup, insert_game_telemetry_manager)
            .add_systems(
                Update,
                (
                    insert_new_game_telemetry
                        .run_if(on_real_timer(Duration::from_secs(DATA_UPLOAD_INTERVAL))),
                    add_progression_core_to_telemetry_manager.run_if(on_real_timer(
                        Duration::from_secs(PROGRESSION_CORE_INTERVAL),
                    )),
                    add_telemetry_actions.after(PlayerMovementSystemSet),
                    send_data_to_server
                        .run_if(on_real_timer(Duration::from_secs(DATA_UPLOAD_INTERVAL))),
                )
                    .chain()
                    .run_if(resource_exists::<GameTelemetryManager>),
            );
    }
}
