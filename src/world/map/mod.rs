mod border;
mod building;
mod debug;
mod flora;
mod grass;

use std::{collections::HashMap, time::Duration};

pub use building::{Blueprint, BuildingSystemSet};
pub use flora::Flora;

use bevy::{prelude::*, time::common_conditions::on_timer};
use flora::FloraData;
use grass::CutTallGrass;
use serde::{Deserialize, Serialize};

use crate::{
    assets::FLORA_DATA_CORE,
    player::{GamingInput, Player},
    ui::ItemPressed,
    BachelorBuild, GameState,
};

use super::{collisions::intersection_aabb_circle, DynamicCollider, TILE_SIZE};

pub const MAP_SIZE: usize = 50;
const EMPTY_CELL_VALUE: u16 = u16::MAX;
const PLAYER_BLOCKED_CELL_VALUE: u16 = u16::MAX - 1;
const TALL_GRASS_CELL_VALUE: u16 = u16::MAX - 2;

const CUT_TALL_GRASS_POINTS: u64 = 1;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            flora::MapFloraPlugin,
            debug::MapDebugPlugin,
            building::MapBuildingPlugin,
            border::MapBorderPlugin,
            grass::MapGrassPlugin,
        ))
        .add_event::<ItemBought>()
        .add_systems(
            OnExit(GameState::AssetLoading),
            (insert_progression_core, insert_map_data_resource),
        )
        .add_systems(
            Update,
            (
                increase_points_on_cut_tall_grass,
                update_map_data_on_tall_grass_cut,
                block_grid_player_pos,
                trigger_item_bought_on_item_pressed.run_if(resource_exists::<BachelorBuild>),
                trigger_item_bought_on_blueprint_build.run_if(resource_exists::<BachelorBuild>),
                update_progression_core_on_item_bought,
                update_points_per_second,
                add_points.run_if(on_timer(Duration::from_secs(1))),
                save_game_state,
            )
                .chain()
                .in_set(ProgressionSystemSet)
                .run_if(resource_exists::<MapData>.and(resource_exists::<ProgressionCore>)),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProgressionSystemSet;

#[derive(Resource, Serialize, Deserialize)]
pub struct ProgressionCore {
    previous_timestamp: u64,
    pub points: u64,
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
    // Background,
    Floor,
    // GroundEffect,
    // BottomEnvironment,
    TopEnvironment,
    TopUi,
}

#[derive(Event)]
pub struct ItemBought {
    pos: Vec2,
    item: Flora,
}

impl ProgressionCore {
    fn empty() -> Self {
        Self {
            previous_timestamp: 0,
            points: 0,
            pps: 0,
            flora: vec![0; Flora::len()],
        }
    }

    pub fn is_affordable(&self, map_data: &MapData, flora: &Flora) -> bool {
        self.points >= map_data.flora_data(flora.index()).cost as u64
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
            grid: vec![[EMPTY_CELL_VALUE; MAP_SIZE]; MAP_SIZE],
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
                if self.grid_index(x, y) == EMPTY_CELL_VALUE {
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

        (x.round() as usize, y.round() as usize)
    }

    fn grid_indices_to_pos(&self, x: usize, y: usize) -> Vec2 {
        Vec2::new(
            x as f32 * TILE_SIZE - (MAP_SIZE / 2) as f32 * TILE_SIZE,
            y as f32 * TILE_SIZE - (MAP_SIZE / 2) as f32 * TILE_SIZE,
        )
    }

    pub fn grid_index(&self, x: usize, y: usize) -> u16 {
        self.grid[x.min(MAP_SIZE - 1)][y.min(MAP_SIZE - 1)]
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

    fn fits_at_grid_position(&self, x: usize, y: usize, x_size: usize, y_size: usize) -> bool {
        if self.grid_index(x, y) != EMPTY_CELL_VALUE {
            return false;
        }

        for inner_x in 0..x_size {
            for inner_y in 0..y_size {
                if self.grid_index(x + inner_x, y + inner_y) != EMPTY_CELL_VALUE {
                    return false;
                }
            }
        }
        true
    }

    fn fits_at_pos(&self, pos: Vec2, object_size: (usize, usize)) -> bool {
        let (x, y) = self.pos_to_grid_indices(pos);
        let (x_size, y_size) = object_size;
        self.fits_at_grid_position(x, y, x_size, y_size)
    }

    fn set_map_data_value_at_pos(
        &mut self,
        bottom_left_corner_pos: Vec2,
        object_size: (usize, usize),
        value: u16,
    ) {
        let (x, y) = self.pos_to_grid_indices(bottom_left_corner_pos);
        let (x_size, y_size) = object_size;

        debug_assert!(self.fits_at_grid_position(x, y, x_size, y_size));

        for inner_x in 0..x_size {
            for inner_y in 0..y_size {
                self.grid[x + inner_x][y + inner_y] = value;
            }
        }
    }

    /// Sets the value at the position to empty.
    fn reset_map_data_at_pos(&mut self, pos: Vec2) {
        let (x, y) = self.pos_to_grid_indices(pos);
        self.grid[x][y] = EMPTY_CELL_VALUE;
    }
}

impl ZLevel {
    pub fn value(&self) -> f32 {
        match self {
            // ZLevel::Background => -1e5,
            ZLevel::Floor => -3e4,
            // ZLevel::GroundEffect => -2e4,
            // ZLevel::BottomEnvironment => -1e4,
            ZLevel::TopEnvironment => 1e4,
            ZLevel::TopUi => 1e4 + 301.0,
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn insert_map_data_resource_wasm(commands: &mut Commands) {
    use crate::assets::WASM_MAP_DATA_KEY_STORAGE;

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

fn insert_map_data_resource(mut commands: Commands) {
    #[cfg(target_arch = "wasm32")]
    insert_map_data_resource_wasm(&mut commands);

    #[cfg(not(target_arch = "wasm32"))]
    commands.insert_resource(MapData::empty());
}

#[cfg(target_arch = "wasm32")]
fn insert_progression_core_wasm(commands: &mut Commands) {
    use crate::assets::WASM_PROGRESSION_CORE_KEY_STORAGE;

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

fn insert_progression_core(mut commands: Commands) {
    #[cfg(target_arch = "wasm32")]
    insert_progression_core_wasm(&mut commands);

    #[cfg(not(target_arch = "wasm32"))]
    commands.insert_resource(ProgressionCore::empty());
}

#[cfg(target_arch = "wasm32")]
fn save_game_state_wasm(core: &ProgressionCore, map_data: &MapData) {
    use crate::assets::{WASM_MAP_DATA_KEY_STORAGE, WASM_PROGRESSION_CORE_KEY_STORAGE};

    use web_sys::window;

    let storage = window()
        .expect("failed to get window")
        .local_storage()
        .expect("failed to get local storage")
        .expect("failed to unwrap local storage");

    storage
        .set_item(WASM_MAP_DATA_KEY_STORAGE, &map_data.to_string())
        .expect("failed to set local storage progression core");

    storage
        .set_item(
            WASM_PROGRESSION_CORE_KEY_STORAGE,
            &serde_json::to_string(core).expect("failed to serialize progression core"),
        )
        .expect("failed to set local storage progression core");
}

fn save_game_state(core: Res<ProgressionCore>, map_data: Res<MapData>) {
    #[cfg(target_arch = "wasm32")]
    save_game_state_wasm(&core, &map_data);
}

fn update_points_per_second(mut core: ResMut<ProgressionCore>, map_data: Res<MapData>) {
    let mut pps = 0;
    for i in 0..core.flora.len() {
        if core.flora[i] == 0 {
            continue;
        }

        pps += core.flora[i] as u32 * map_data.flora_data(i).pps;
    }

    core.pps = pps;
}

fn add_points(mut core: ResMut<ProgressionCore>) {
    core.points += core.pps as u64;
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
        debug_assert!(core.points >= map_data.flora_data(ev.item.index()).cost as u64);

        core.flora[ev.item.index()] += 1;
        core.points -= (map_data.flora_data(ev.item.index()).cost as u64).min(core.points);
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
        map_data.reset_map_data_at_pos(ev.pos);
    }
}

fn increase_points_on_cut_tall_grass(
    mut core: ResMut<ProgressionCore>,
    mut ev_cut_tall_grass: EventReader<CutTallGrass>,
) {
    for _ in ev_cut_tall_grass.read() {
        core.points += CUT_TALL_GRASS_POINTS;
    }
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
