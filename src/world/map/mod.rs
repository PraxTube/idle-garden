mod border;
mod building;
mod clouds;
mod debug;
mod flora;
mod grass;
mod telemetry;

#[cfg(not(target_arch = "wasm32"))]
use std::fs::{self, read_to_string};
use std::{collections::HashMap, time::Duration};

use telemetry::GameTelemetryManager;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub use building::{Blueprint, BuildingSystemSet};
pub use flora::{Flora, InitialFloraSpawned};

use bevy::{
    prelude::*,
    time::common_conditions::{on_real_timer, on_timer},
};

#[cfg(not(target_arch = "wasm32"))]
use crate::assets::{GAME_TELEMETRY_FILE, MAP_DATA_FILE, PLAYER_SAVE_FILE, PROGRESSION_CORE_FILE};
#[cfg(target_arch = "wasm32")]
use crate::assets::{WASM_KEYS, WASM_MAP_DATA_KEY_STORAGE, WASM_PROGRESSION_CORE_KEY_STORAGE};

use flora::FloraData;
use grass::CutTallGrass;
use serde::{Deserialize, Serialize};

use crate::{
    assets::FLORA_DATA_CORE,
    player::{GamingInput, Player},
    ui::{ItemPressed, MenuAction, MenuActionEvent},
    BachelorBuild,
};

use super::{collisions::intersection_aabb_circle, DynamicCollider, TILE_SIZE};

pub const MAP_SIZE: usize = 80;
const EMPTY_CELL_VALUE: u16 = u16::MAX;
const PLAYER_BLOCKED_CELL_VALUE: u16 = u16::MAX - 1;
const TALL_GRASS_CELL_VALUE: u16 = u16::MAX - 2;

const DEFAULT_POINTS_CAP: u64 = 800;
const POINTS_CAP_INCEASE_PER_SILO: u64 = 300;
const AUTO_SAVE_TIME_INTERVAL: u64 = 60;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            debug::MapDebugPlugin,
            border::MapBorderPlugin,
            building::MapBuildingPlugin,
            clouds::CloudsPlugin,
            flora::MapFloraPlugin,
            grass::MapGrassPlugin,
            telemetry::GameTelemetryPlugin,
        ))
        .add_event::<ItemBought>()
        .add_event::<AutoSave>()
        .add_systems(Startup, (insert_progression_core, insert_map_data_resource))
        .add_systems(
            PreUpdate,
            (
                add_offline_progression.run_if(
                    resource_exists::<ProgressionCore>
                        .and(resource_exists::<MapData>)
                        .and(run_once),
                ),
                // We also check if MapData exists (even though we don't need it in the system
                // itself). The reason for that is because we need the MapData for the calculation
                // of how much idle progress we made during the timestamp diff.
                // And we can't have this system run before we calculate it because it will
                // obviuosly get overwritten otherwise.
                update_progression_core_timestamp.run_if(
                    resource_exists::<ProgressionCore>
                        .and(resource_exists::<MapData>)
                        .and(on_real_timer(Duration::from_secs(1))),
                ),
                trigger_auto_save
                    .run_if(on_real_timer(Duration::from_secs(AUTO_SAVE_TIME_INTERVAL))),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                update_map_data_on_tall_grass_cut,
                block_grid_player_pos,
                trigger_item_bought_on_item_pressed.run_if(resource_exists::<BachelorBuild>),
                trigger_item_bought_on_blueprint_build.run_if(resource_exists::<BachelorBuild>),
                update_progression_core_on_item_bought,
                update_points_per_second,
                update_points_cap,
                add_points.run_if(on_timer(Duration::from_secs(1))),
                reset_game_state,
                save_game_state.run_if(on_event::<AutoSave>),
            )
                .chain()
                .in_set(ProgressionSystemSet)
                .run_if(resource_exists::<MapData>.and(resource_exists::<ProgressionCore>)),
        );

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            PostUpdate,
            save_game_state.run_if(
                resource_exists::<MapData>
                    .and(resource_exists::<ProgressionCore>.and(on_event::<AppExit>)),
            ),
        );
        #[cfg(target_arch = "wasm32")]
        app.add_systems(
            Update,
            sync_state_to_js.run_if(resource_exists::<MapData>.and(
                resource_exists::<ProgressionCore>.and(on_real_timer(Duration::from_secs(1))),
            )),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProgressionSystemSet;

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct ProgressionCore {
    previous_timestamp: u64,
    /// We store the points generate while offline in here.
    /// This is also used as a flag, if there was progress done offline,
    /// meaning this value is >0, than we spawn a number pop up indicating that
    /// and than reset this value back to 0.
    ///
    /// Don't use this value for anything else.
    offline_progression: u64,
    pub points: u64,
    pub points_cap: u64,
    pub pps: u32,
    pub flora: Vec<u16>,
}

#[derive(Resource)]
pub struct MapData {
    grid: Vec<[u16; MAP_SIZE]>,
    flora_data: Vec<FloraData>,
    previous_player_blocked_cells: Vec<(usize, usize)>,
}

pub enum ZLevel {
    Background,
    // Floor,
    // GroundEffect,
    // BottomEnvironment,
    TopEnvironment,
    TopUi,
}

#[derive(Event)]
pub struct ItemBought {
    pub pos: Vec2,
    pub cost: u32,
    item: Flora,
}

#[derive(Event)]
pub struct AutoSave;

impl ProgressionCore {
    fn empty() -> Self {
        Self {
            previous_timestamp: 0,
            offline_progression: 0,
            points: 0,
            points_cap: DEFAULT_POINTS_CAP,
            pps: 0,
            flora: vec![0; Flora::len()],
        }
    }

    pub fn is_affordable(&self, map_data: &MapData, flora: &Flora) -> bool {
        self.points
            >= map_data
                .flora_data(flora.index())
                .cost(self.flora[flora.index()].into()) as u64
    }

    fn update_points_cap(&mut self) {
        self.points_cap = DEFAULT_POINTS_CAP
            + self.flora[Flora::Silo.index()] as u64 * POINTS_CAP_INCEASE_PER_SILO;
    }
}

impl MapData {
    fn build_flora_data() -> Vec<FloraData> {
        let map: HashMap<Flora, FloraData> = serde_json::from_str(FLORA_DATA_CORE)
            .expect("failed to parse flora data core to json str");

        debug_assert_eq!(map.len(), Flora::len());

        let mut data = vec![FloraData::default(); Flora::len()];

        for key in map.keys() {
            data[key.index()] = map
                .get(key)
                .expect("failed to get key, must never happen")
                .clone();
        }

        data
    }

    fn empty() -> Self {
        Self {
            grid: vec![[TALL_GRASS_CELL_VALUE; MAP_SIZE]; MAP_SIZE],
            flora_data: Self::build_flora_data(),
            previous_player_blocked_cells: Vec::new(),
        }
    }

    /// Expects string to be of form
    ///
    /// usize,usize:u16;REPEAT
    fn from_str(string: &str) -> Self {
        let mut map_data = MapData::empty();

        if string.is_empty() {
            return map_data;
        }

        for raw_data_point in string.split(';').collect::<Vec<&str>>() {
            let parts = raw_data_point.split(':').collect::<Vec<&str>>();
            debug_assert_eq!(parts.len(), 2, "{:?}", string);

            let xy = parts[0].split(',').collect::<Vec<&str>>();

            debug_assert_eq!(xy.len(), 2, "{:?}", string);

            let (x, y) = (
                xy[0].parse::<usize>().expect("failed to parse to usize"),
                xy[1].parse::<usize>().expect("failed to parse to usize"),
            );

            let value = parts[1]
                .parse::<u16>()
                .expect("failed to parse value to u16");

            map_data.grid[x][y] = value;
        }

        map_data
    }

    fn to_string(&self) -> String {
        let mut string = String::new();

        for x in 0..MAP_SIZE {
            for y in 0..MAP_SIZE {
                if self.grid_index(x, y) == EMPTY_CELL_VALUE
                    || self.grid_index(x, y) == PLAYER_BLOCKED_CELL_VALUE
                {
                    continue;
                }

                if !string.is_empty() {
                    string.push(';');
                }

                string.push_str(&format!("{},{}:{}", x, y, self.grid_index(x, y)));
            }
        }
        string
    }

    fn pos_to_grid_indices(&self, p: Vec2) -> (usize, usize) {
        let p = p / TILE_SIZE + Vec2::ONE * 0.5 * MAP_SIZE as f32;

        let x = p.x.clamp(0.0, MAP_SIZE as f32 - 1.0);
        let y = p.y.clamp(0.0, MAP_SIZE as f32 - 1.0);

        self.clamp_indices(x.round() as usize, y.round() as usize)
    }

    fn grid_indices_to_pos(&self, x: usize, y: usize) -> Vec2 {
        Vec2::new(
            x as f32 * TILE_SIZE - (MAP_SIZE / 2) as f32 * TILE_SIZE,
            y as f32 * TILE_SIZE - (MAP_SIZE / 2) as f32 * TILE_SIZE,
        )
    }

    fn clamp_indices(&self, x: usize, y: usize) -> (usize, usize) {
        (x.min(MAP_SIZE - 1), y.min(MAP_SIZE - 1))
    }

    pub fn grid_index(&self, x: usize, y: usize) -> u16 {
        let (clamped_x, clamped_y) = self.clamp_indices(x, y);
        self.grid[clamped_x][clamped_y]
    }

    pub fn indices_in_grid(&self, x: usize, y: usize) -> bool {
        x < MAP_SIZE && y < MAP_SIZE
    }

    pub fn flora_data(&self, index: usize) -> FloraData {
        debug_assert!(index < self.flora_data.len());

        if index >= self.flora_data.len() {
            error!("attempted to get flora data from index out of range, must never happen!");
            return FloraData::default();
        }

        self.flora_data[index].clone()
    }

    /// Return whether the cells at the given position with the size are all empty or grass.
    fn fits_at_empty_or_grass_position(
        &self,
        x: usize,
        y: usize,
        x_size: usize,
        y_size: usize,
    ) -> bool {
        debug_assert_ne!(x_size, 0);
        debug_assert_ne!(y_size, 0);

        for inner_x in 0..x_size {
            for inner_y in 0..y_size {
                let index = self.grid_index(x + inner_x, y + inner_y);
                if index != EMPTY_CELL_VALUE && index != TALL_GRASS_CELL_VALUE {
                    return false;
                }
            }
        }
        true
    }

    fn fits_at_pos(&self, pos: Vec2, object_size: (usize, usize)) -> bool {
        let (x, y) = self.pos_to_grid_indices(pos);
        let (x_size, y_size) = object_size;
        self.fits_at_empty_or_grass_position(x, y, x_size, y_size)
    }

    fn set_map_data_value_at_pos(
        &mut self,
        bottom_left_corner_pos: Vec2,
        object_size: (usize, usize),
        value: u16,
    ) {
        let (x, y) = self.pos_to_grid_indices(bottom_left_corner_pos);
        let (x_size, y_size) = object_size;

        debug_assert!(
            self.fits_at_empty_or_grass_position(x, y, x_size, y_size),
            "value: {}, pos: {}, (x, y): {}, {}, grid: {}",
            value,
            bottom_left_corner_pos,
            x,
            y,
            self.grid[x][y]
        );

        for inner_x in 0..x_size {
            for inner_y in 0..y_size {
                let (clamped_x, clamped_y) = self.clamp_indices(x + inner_x, y + inner_y);
                self.grid[clamped_x][clamped_y] = value;
            }
        }
    }

    /// Sets the value at the position to empty. Only works when the current index is tall grass.
    fn set_tall_grass_cell_value_to_empty(&mut self, pos: Vec2) {
        let (x, y) = self.pos_to_grid_indices(pos);
        if self.grid_index(x, y) != TALL_GRASS_CELL_VALUE {
            return;
        }
        self.grid[x][y] = EMPTY_CELL_VALUE;
    }
}

impl ZLevel {
    pub fn value(&self) -> f32 {
        match self {
            ZLevel::Background => -1e5,
            // ZLevel::Floor => -3e4,
            // ZLevel::GroundEffect => -2e4,
            // ZLevel::BottomEnvironment => -1e4,
            ZLevel::TopEnvironment => 1e4,
            ZLevel::TopUi => 1e4 + 301.0,
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn insert_map_data_wasm(commands: &mut Commands) {
    use web_sys::window;

    let storage = window()
        .expect("failed to get window")
        .local_storage()
        .expect("failed to get local storage")
        .expect("failed to unwrap local storage");

    let map_data = match storage
        .get_item(WASM_MAP_DATA_KEY_STORAGE)
        .expect("failed to get local storage item WASM key")
    {
        Some(r) => MapData::from_str(&r),
        None => MapData::empty(),
    };

    commands.insert_resource(map_data);
}

#[cfg(not(target_arch = "wasm32"))]
fn insert_map_data_native(commands: &mut Commands) {
    let raw_map_data = read_to_string(MAP_DATA_FILE).expect("failed to read map data file");
    let map_data = MapData::from_str(&raw_map_data);
    commands.insert_resource(map_data);
}

fn insert_map_data_resource(mut commands: Commands) {
    #[cfg(target_arch = "wasm32")]
    insert_map_data_wasm(&mut commands);

    #[cfg(not(target_arch = "wasm32"))]
    insert_map_data_native(&mut commands);
}

#[cfg(target_arch = "wasm32")]
fn insert_progression_core_wasm(commands: &mut Commands) {
    use web_sys::window;

    let storage = window()
        .expect("failed to get window")
        .local_storage()
        .expect("failed to get local storage")
        .expect("failed to unwrap local storage");

    let core = match storage
        .get_item(WASM_PROGRESSION_CORE_KEY_STORAGE)
        .expect("failed to get local storage item WASM key")
    {
        Some(r) => {
            serde_json::from_str(&r).expect("failed to parse progression core from json string")
        }
        None => ProgressionCore::empty(),
    };

    commands.insert_resource(core);
}

#[cfg(not(target_arch = "wasm32"))]
fn insert_progression_core_native(commands: &mut Commands) {
    let raw_core =
        read_to_string(PROGRESSION_CORE_FILE).expect("failed to read progression core file");
    let core = if raw_core.is_empty() {
        ProgressionCore::empty()
    } else {
        serde_json::from_str(&raw_core).expect("failed to parse progression core from json string")
    };
    commands.insert_resource(core);
}

fn insert_progression_core(mut commands: Commands) {
    #[cfg(target_arch = "wasm32")]
    insert_progression_core_wasm(&mut commands);

    #[cfg(not(target_arch = "wasm32"))]
    insert_progression_core_native(&mut commands);
}

#[cfg(target_arch = "wasm32")]
fn save_game_state_wasm(keys: &[&str; 4], data: &[String; 4]) {
    use web_sys::window;

    let storage = window()
        .expect("failed to get window")
        .local_storage()
        .expect("failed to get local storage")
        .expect("failed to unwrap local storage");

    for i in 0..keys.len() {
        let err_msg = format!(
            "failed to store to local storage, key: {}, data: {}",
            keys[i], &data[i]
        );

        storage.set_item(keys[i], &data[i]).expect(&err_msg);
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn buffer_game_state(json: &str);
}

#[cfg(target_arch = "wasm32")]
fn sync_state_to_js(
    core: Res<ProgressionCore>,
    map_data: Res<MapData>,
    telemetry: Res<GameTelemetryManager>,
    q_player: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = q_player.single() else {
        return;
    };

    let data = package_save_data(&core, &map_data, &telemetry, &player_transform);
    let save_data: HashMap<&&str, &String> = WASM_KEYS.iter().zip(data.iter()).collect();

    buffer_game_state(
        &serde_json::to_string(&save_data).expect("failed to parse hashmap save data to json"),
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn save_game_state_native(data: &[String; 4]) {
    const FILES: [&str; 4] = [
        PROGRESSION_CORE_FILE,
        MAP_DATA_FILE,
        PLAYER_SAVE_FILE,
        GAME_TELEMETRY_FILE,
    ];

    for i in 0..FILES.len() {
        let err_msg = format!(
            "failed to write data to file, file: {}, data: {}",
            FILES[i], data[i]
        );

        fs::write(FILES[i], &data[i]).expect(&err_msg);
    }
}

/// 1. Core
/// 2. MapData
/// 3. Player Pos
/// 4. Game Telemetry
fn package_save_data(
    core: &ProgressionCore,
    map_data: &MapData,
    telemetry: &GameTelemetryManager,
    player_transform: &Transform,
) -> [String; 4] {
    [
        serde_json::to_string(core).expect("failed to serialize progression core"),
        map_data.to_string(),
        format!(
            "{},{}",
            player_transform.translation.x, player_transform.translation.y
        ),
        serde_json::to_string(telemetry).expect("failed to serialize game telemetry"),
    ]
}

fn trigger_auto_save(mut ev_auto_save: EventWriter<AutoSave>) {
    ev_auto_save.write(AutoSave);
}

fn save_game_state(
    core: Res<ProgressionCore>,
    map_data: Res<MapData>,
    telemetry: Res<GameTelemetryManager>,
    q_player: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = q_player.single() else {
        return;
    };

    let data = package_save_data(&core, &map_data, &telemetry, &player_transform);

    #[cfg(target_arch = "wasm32")]
    save_game_state_wasm(&WASM_KEYS, &data);

    #[cfg(not(target_arch = "wasm32"))]
    save_game_state_native(&data);
}

fn reset_game_state(
    mut core: ResMut<ProgressionCore>,
    mut map_data: ResMut<MapData>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    if !ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::Reset)
    {
        return;
    }

    *core = ProgressionCore::empty();
    *map_data = MapData::empty();
}

fn compute_current_pps(core: &ProgressionCore, map_data: &MapData) -> u32 {
    let mut pps = 0;
    for i in 0..core.flora.len() {
        if core.flora[i] == 0 {
            continue;
        }

        pps += core.flora[i] as u32 * map_data.flora_data(i).pps;
    }
    pps
}

fn update_points_per_second(mut core: ResMut<ProgressionCore>, map_data: Res<MapData>) {
    let pps = compute_current_pps(&core, &map_data);
    core.pps = pps;
}

fn update_points_cap(mut core: ResMut<ProgressionCore>) {
    core.update_points_cap();
}

fn add_points(mut core: ResMut<ProgressionCore>) {
    core.points = (core.points + core.pps as u64).min(core.points_cap);
}

fn trigger_item_bought_on_item_pressed(
    core: Res<ProgressionCore>,
    map_data: Res<MapData>,
    bachelor_build: Res<BachelorBuild>,
    mut ev_item_pressed: EventReader<ItemPressed>,
    mut ev_item_bought: EventWriter<ItemBought>,
) {
    if bachelor_build.with_building {
        return;
    }

    for ev in ev_item_pressed.read() {
        if core.is_affordable(&map_data, &ev.flora) {
            ev_item_bought.write(ItemBought {
                pos: Vec2::ZERO,
                cost: map_data.flora_data[ev.flora.index()]
                    .cost(core.flora[ev.flora.index()].into()),
                item: ev.flora,
            });
        }
    }
}

fn trigger_item_bought_on_blueprint_build(
    core: Res<ProgressionCore>,
    map_data: Res<MapData>,
    gaming_input: Res<GamingInput>,
    bachelor_build: Res<BachelorBuild>,
    q_player: Query<&Player>,
    q_blueprint: Query<(&Transform, &Blueprint)>,
    mut ev_item_bought: EventWriter<ItemBought>,
) {
    if !bachelor_build.with_building {
        return;
    }

    if !gaming_input.confirm {
        return;
    }

    let Ok(player) = q_player.single() else {
        return;
    };
    if player.is_over_ui {
        return;
    };

    let Ok((transform, blueprint)) = q_blueprint.single() else {
        return;
    };

    if !blueprint.fits_at_pos {
        return;
    }

    if core.is_affordable(&map_data, &blueprint.item) {
        ev_item_bought.write(ItemBought {
            pos: transform.translation.xy(),
            cost: map_data.flora_data[blueprint.item.index()]
                .cost(core.flora[blueprint.item.index()].into()),
            item: blueprint.item,
        });
    }
}

fn update_progression_core_on_item_bought(
    mut core: ResMut<ProgressionCore>,
    map_data: Res<MapData>,
    mut ev_item_bought: EventReader<ItemBought>,
) {
    for ev in ev_item_bought.read() {
        let cost = map_data
            .flora_data(ev.item.index())
            .cost(core.flora[ev.item.index()].into()) as u64;

        debug_assert!(core.points >= cost);
        core.points -= cost.min(core.points);
        core.flora[ev.item.index()] += 1;
    }
}

fn block_grid_player_pos(
    mut map_data: ResMut<MapData>,
    q_player: Query<(&Transform, &DynamicCollider), With<Player>>,
) {
    let Ok((transform, collider)) = q_player.single() else {
        return;
    };

    for (x, y) in map_data.previous_player_blocked_cells.clone() {
        map_data.grid[x][y] = EMPTY_CELL_VALUE;
    }

    let pos = transform.translation.xy() + collider.offset;

    let mut blocked_cells = Vec::new();
    for x in -1..=1 {
        for y in -1..=1 {
            let offset = Vec2::new(TILE_SIZE * x as f32, TILE_SIZE * y as f32);
            let (offset_x, offset_y) = map_data.pos_to_grid_indices(pos + offset);
            let tile_pos = map_data.grid_indices_to_pos(offset_x, offset_y);

            if !intersection_aabb_circle(
                collider.radius,
                pos,
                0.5 * TILE_SIZE * Vec2::ONE,
                tile_pos,
            ) {
                continue;
            }

            if map_data.grid[offset_x][offset_y] == EMPTY_CELL_VALUE {
                map_data.grid[offset_x][offset_y] = PLAYER_BLOCKED_CELL_VALUE;
                blocked_cells.push((offset_x, offset_y));
            }
        }
    }
    map_data.previous_player_blocked_cells = blocked_cells;
}

fn update_map_data_on_tall_grass_cut(
    mut map_data: ResMut<MapData>,
    mut ev_cut_tall_grass: EventReader<CutTallGrass>,
) {
    for ev in ev_cut_tall_grass.read() {
        map_data.set_tall_grass_cell_value_to_empty(ev.pos);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn timestamp_native() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards, have you alterated the system time?");
    duration.as_secs()
}

#[cfg(target_arch = "wasm32")]
fn timestamp_wasm() -> u64 {
    use js_sys::Date;
    (Date::now() * 0.001) as u64
}

fn timestamp() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    let timestamp = timestamp_native();

    #[cfg(target_arch = "wasm32")]
    let timestamp = timestamp_wasm();

    timestamp
}

fn update_progression_core_timestamp(mut core: ResMut<ProgressionCore>) {
    #[cfg(not(target_arch = "wasm32"))]
    let timestamp = timestamp_native();

    #[cfg(target_arch = "wasm32")]
    let timestamp = timestamp_wasm();

    core.previous_timestamp = timestamp;
}

fn add_offline_progression(mut core: ResMut<ProgressionCore>, map_data: Res<MapData>) {
    let timestamp = timestamp();

    debug_assert!(timestamp > core.previous_timestamp);

    if core.previous_timestamp > timestamp {
        error!("The previous timestamp is greater than the current timestamp, did you alter the systems time?");
        return;
    }

    let diff = timestamp - core.previous_timestamp;
    let pps = compute_current_pps(&core, &map_data) as u64;

    debug_assert!(core.points <= core.points_cap);
    if core.points == core.points_cap {
        core.offline_progression = 0;
        return;
    }

    let new_points = (pps * diff).min(core.points_cap - core.points);
    core.points += new_points;
    core.offline_progression = new_points;
}

#[cfg(debug_assertions)]
pub fn simulate_progression() {
    fn flora_evaluation(cost: u64, pps: u32) -> f32 {
        if cost == 0 {
            return f32::MAX;
        }

        (pps as f32) / (cost as f32)
    }

    fn get_next_item_index(core: &ProgressionCore, flora_data: &[FloraData]) -> Option<usize> {
        if core.points
            >= flora_data[Flora::Silo.index()].cost(core.flora[Flora::Silo.index()].into()) as u64
        {
            return Some(Flora::Silo.index());
        }

        let mut best_index = usize::MAX;
        let mut best_evaluation = f32::NEG_INFINITY;
        for i in 0..core.flora.len() {
            let cost = flora_data[i].cost(core.flora[i].into()) as u64;
            if core.points < cost {
                continue;
            }

            let evaluation = flora_evaluation(cost, flora_data[i].pps);
            if evaluation > best_evaluation {
                best_index = i;
                best_evaluation = evaluation;
            }
        }

        if best_index == usize::MAX {
            return None;
        }
        Some(best_index)
    }

    let mut core = ProgressionCore::empty();
    let map_data = MapData::empty();
    assert_eq!(core.flora.len(), map_data.flora_data.len());

    core.points = DEFAULT_POINTS_CAP / 10;
    assert!(core.points > 0);

    let mut data = Vec::new();

    const MAX_TICKS: usize = 1000;
    for time in 0..MAX_TICKS {
        data.push((time, core.clone()));

        let pps = compute_current_pps(&core, &map_data);
        core.pps = pps;
        core.update_points_cap();
        core.points = (core.points + core.pps as u64).min(core.points_cap);

        let Some(index) = get_next_item_index(&core, &map_data.flora_data) else {
            continue;
        };

        let cost = map_data.flora_data[index].cost(core.flora[index].into()) as u64;
        assert!(core.points >= cost);
        core.points -= cost;
        core.flora[index] += 1;
    }

    let content = data
        .iter()
        .map(|(time, core)| {
            format!(
                "{}:{};{};{};[{}]",
                time,
                core.points,
                core.points_cap,
                core.pps,
                core.flora
                    .iter()
                    .map(|f| format!("{}", f))
                    .collect::<Vec<String>>()
                    .join(",")
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    std::fs::write("SIMULATION_PROGRESS_OUT.csv", content).expect("failed to write to file");

    println!("{}", core.points);
    println!("{}", core.points_cap);
    println!("{}", core.pps);

    println!("{:?}", core.flora);
}

#[test]
fn validate_pos_to_grid_indices() {
    let map_data = MapData::empty();

    assert_eq!(
        map_data.pos_to_grid_indices(Vec2::ZERO),
        (MAP_SIZE / 2, MAP_SIZE / 2)
    );

    assert_eq!(
        map_data.pos_to_grid_indices(-Vec2::ONE * TILE_SIZE * MAP_SIZE as f32),
        (0, 0)
    );

    assert_eq!(
        map_data.pos_to_grid_indices(Vec2::ONE * TILE_SIZE * MAP_SIZE as f32),
        (MAP_SIZE - 1, MAP_SIZE - 1)
    );

    assert_eq!(
        map_data.pos_to_grid_indices(Vec2::ONE * TILE_SIZE),
        (MAP_SIZE / 2 + 1, MAP_SIZE / 2 + 1)
    );

    assert_eq!(
        map_data.pos_to_grid_indices(
            -Vec2::ONE * 0.5 * TILE_SIZE * MAP_SIZE as f32 + Vec2::ONE * TILE_SIZE
        ),
        (1, 1)
    );
}

#[test]
fn validate_grid_indices_to_pos() {
    let map_data = MapData::empty();

    assert_eq!(
        map_data.grid_indices_to_pos(0, 0),
        -Vec2::ONE * TILE_SIZE * 0.5 * MAP_SIZE as f32
    );
    assert_eq!(
        map_data.grid_indices_to_pos(MAP_SIZE / 2, MAP_SIZE / 2),
        Vec2::ZERO
    );
    assert_eq!(
        map_data.grid_indices_to_pos(MAP_SIZE / 2 + 1, MAP_SIZE / 2),
        Vec2::new(TILE_SIZE, 0.0)
    );
}

#[test]
fn validate_grid_index() {
    let map_data = MapData::empty();

    map_data.grid_index(0, 0);
    map_data.grid_index(MAP_SIZE, 0);
    map_data.grid_index(MAP_SIZE + 100, 0);
    map_data.grid_index(MAP_SIZE + 100, MAP_SIZE);
    map_data.grid_index(MAP_SIZE + 100, MAP_SIZE + 100);
}

#[test]
fn validate_flora_len_matches_json_data() {
    let map: HashMap<Flora, FloraData> =
        serde_json::from_str(FLORA_DATA_CORE).expect("failed to parse flora data core to json str");

    assert_eq!(Flora::len(), map.len());
}

#[test]
fn validate_silo_points_cap_increases_fast_enough() {
    let map_data = MapData::empty();

    assert!(
        POINTS_CAP_INCEASE_PER_SILO
            >= (map_data.flora_data[Flora::Silo.index()].cost_growth_factor).round() as u64
    )
}
