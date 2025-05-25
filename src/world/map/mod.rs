mod debug;
mod flora;

pub use flora::Flora;

use bevy::prelude::*;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{GameAssets, GameState};

use super::TILE_SIZE;

const MAP_SIZE: usize = 500;
const MAX_RANDOM_SEARCH_TRIES: usize = 50;
const GRID_SEARCH_SIZE: usize = 10;
/// Amount of tiles the searching will avoid around the player. Inclusive.
const GRID_PLAYER_PADDING: usize = 2;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((flora::MapFloraPlugin, debug::MapDebugPlugin))
            .add_systems(OnExit(GameState::AssetLoading), spawn_grass)
            .add_systems(
                OnExit(GameState::AssetLoading),
                (insert_progression_core, insert_map_grid_resource),
            )
            .add_systems(
                Update,
                save_game_state
                    .run_if(resource_exists::<MapGrid>.and(resource_exists::<ProgressionCore>)),
            );
    }
}

#[derive(Resource, Serialize, Deserialize)]
pub struct ProgressionCore {
    previous_timestamp: u64,
    pub flora: Vec<u16>,
}

#[derive(Resource)]
pub struct MapGrid {
    grid: Vec<[u16; MAP_SIZE]>,
}

pub enum ZLevel {
    // Background,
    Floor,
    // GroundEffect,
    // BottomEnvironment,
    // TopEnvironment,
    TopUi,
}

impl ProgressionCore {
    fn empty() -> Self {
        Self {
            previous_timestamp: 0,
            flora: vec![0; Flora::len()],
        }
    }
}

impl MapGrid {
    fn empty() -> Self {
        Self {
            grid: vec![[u16::MAX; MAP_SIZE]; MAP_SIZE],
        }
    }

    /// Expects string to be of form
    ///
    /// usize,usize:u16;REPEAT
    fn from_str(string: &str) -> Self {
        let mut map_grid = MapGrid::empty();

        if string.is_empty() {
            return map_grid;
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

            map_grid.grid[x][y] = value;
        }

        map_grid
    }

    fn to_string(&self) -> String {
        let mut string = String::new();

        for x in 0..MAP_SIZE {
            for y in 0..MAP_SIZE {
                if self.grid_index(x, y) == u16::MAX {
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

    fn fits_at_grid_position(&self, x: usize, y: usize, x_size: usize, y_size: usize) -> bool {
        if self.grid_index(x, y) != u16::MAX {
            return false;
        }

        for inner_x in 0..x_size {
            for inner_y in 0..y_size {
                if self.grid_index(x + inner_x, y + inner_y) != u16::MAX {
                    return false;
                }
            }
        }
        true
    }

    fn random_grid_position_near_player_pos(
        &self,
        player_pos: Vec2,
        object_size: (usize, usize),
    ) -> Vec2 {
        let (x_size, y_size) = object_size;
        debug_assert!(x_size > 0);
        debug_assert!(y_size > 0);

        let (player_x, player_y) = self.pos_to_grid_indices(player_pos);

        let mut rng = thread_rng();
        for _ in 0..MAX_RANDOM_SEARCH_TRIES {
            let x = if rng.gen_bool(0.5) {
                (player_x + rng.gen_range(GRID_PLAYER_PADDING + 1..GRID_SEARCH_SIZE))
                    .min(MAP_SIZE - 1)
            } else {
                (player_x as i32 - rng.gen_range(GRID_PLAYER_PADDING + 1..GRID_SEARCH_SIZE) as i32)
                    .max(0) as usize
            };

            let y = if rng.gen_bool(0.5) {
                (player_y + rng.gen_range(GRID_PLAYER_PADDING + 1..GRID_SEARCH_SIZE))
                    .min(MAP_SIZE - 1)
            } else {
                (player_y as i32 - rng.gen_range(GRID_PLAYER_PADDING + 1..GRID_SEARCH_SIZE) as i32)
                    .max(0) as usize
            };

            if self.fits_at_grid_position(x, y, x_size, y_size) {
                return self.grid_indices_to_pos(x, y);
            }
        }

        self.grid_position_from_player_pos(player_pos, object_size)
    }

    fn grid_position_from_player_pos(&self, player_pos: Vec2, object_size: (usize, usize)) -> Vec2 {
        let (x_size, y_size) = object_size;
        debug_assert!(x_size > 0);
        debug_assert!(y_size > 0);

        let (player_x, player_y) = self.pos_to_grid_indices(player_pos);
        let start_x = player_x.max(GRID_SEARCH_SIZE) - GRID_SEARCH_SIZE;
        let start_y = player_y.max(GRID_SEARCH_SIZE) - GRID_SEARCH_SIZE;

        for x in 0..2 * GRID_SEARCH_SIZE {
            for y in 0..2 * GRID_SEARCH_SIZE {
                if x >= GRID_SEARCH_SIZE - GRID_PLAYER_PADDING
                    && x <= GRID_SEARCH_SIZE + GRID_PLAYER_PADDING
                    && y > GRID_SEARCH_SIZE - GRID_PLAYER_PADDING
                    && y < GRID_SEARCH_SIZE + GRID_PLAYER_PADDING
                {
                    continue;
                }

                if !self.fits_at_grid_position(start_x + x, start_y + y, x_size, y_size) {
                    continue;
                }

                return self.grid_indices_to_pos(start_x + x, start_y + y);
            }
        }

        // If we don't find any suitable position we just place it at the bottom left.
        // This will obviously result in a bunch of flora adding up in the worst case,
        // but that doesn't matter as the player is very luckily not going to ever see that
        // anyways. And this is also only for the study.
        self.grid_indices_to_pos(0, 0)
    }

    fn set_map_grid_value_at_pos(
        &mut self,
        bottom_left_corner_pos: Vec2,
        object_size: (usize, usize),
        value: u16,
    ) {
        let (x, y) = self.pos_to_grid_indices(bottom_left_corner_pos);
        let (x_size, y_size) = object_size;

        if x == 0 && y == 0 {
            warn!("object at bottom left corner of map (0, 0), can happen, if happens too frequently you have to change something.");
            return;
        }

        debug_assert!(x_size > 0);
        debug_assert!(y_size > 0);
        debug_assert!(self.fits_at_grid_position(x, y, x_size, y_size));

        for inner_x in 0..x_size {
            for inner_y in 0..y_size {
                self.grid[x + inner_x][y + inner_y] = value;
            }
        }
    }
}

impl ZLevel {
    pub fn value(&self) -> f32 {
        match self {
            // ZLevel::Background => -1e5,
            ZLevel::Floor => -3e4,
            // ZLevel::GroundEffect => -2e4,
            // ZLevel::BottomEnvironment => -1e4,
            // ZLevel::TopEnvironment => 1e4,
            ZLevel::TopUi => 1e4 + 301.0,
        }
    }
}

fn spawn_grass(mut commands: Commands, assets: Res<GameAssets>) {
    let mut rng = thread_rng();

    let size = 25;
    for i in -size..size {
        for j in -size..size {
            if rng.gen_range(0..100) < 95 {
                continue;
            }

            let pos = Vec2::new(i as f32 * TILE_SIZE, j as f32 * TILE_SIZE);
            commands.spawn((
                Transform::from_translation(pos.extend(ZLevel::Floor.value())),
                Sprite::from_image(assets.grass.clone()),
            ));
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn insert_map_grid_resource_wasm(commands: &mut Commands) {
    use crate::assets::WASM_MAP_GRID_KEY_STORAGE;

    use web_sys::window;

    let storage = window()
        .expect("failed to get window")
        .local_storage()
        .expect("failed to get local storage")
        .expect("failed to unwrap local storage");

    let map_grid = match storage
        .get_item(WASM_MAP_GRID_KEY_STORAGE)
        .expect("failed to get local storage item WASM key")
    {
        Some(r) => MapGrid::from_str(&r),
        None => MapGrid::empty(),
    };

    commands.insert_resource(map_grid);
}

fn insert_map_grid_resource(mut commands: Commands) {
    #[cfg(target_arch = "wasm32")]
    insert_map_grid_resource_wasm(&mut commands);

    #[cfg(not(target_arch = "wasm32"))]
    commands.insert_resource(MapGrid::empty());
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
fn save_game_state_wasm(core: &ProgressionCore, map_grid: &MapGrid) {
    use crate::assets::{WASM_MAP_GRID_KEY_STORAGE, WASM_PROGRESSION_CORE_KEY_STORAGE};

    use web_sys::window;

    let storage = window()
        .expect("failed to get window")
        .local_storage()
        .expect("failed to get local storage")
        .expect("failed to unwrap local storage");

    storage
        .set_item(WASM_MAP_GRID_KEY_STORAGE, &map_grid.to_string())
        .expect("failed to set local storage progression core");

    storage
        .set_item(
            WASM_PROGRESSION_CORE_KEY_STORAGE,
            &serde_json::to_string(core).expect("failed to serialize progression core"),
        )
        .expect("failed to set local storage progression core");
}

fn save_game_state(core: Res<ProgressionCore>, map_grid: Res<MapGrid>) {
    #[cfg(target_arch = "wasm32")]
    save_game_state_wasm(&core, &map_grid);
}

#[test]
fn validate_pos_to_grid_indices() {
    let map_grid = MapGrid::empty();

    assert_eq!(
        map_grid.pos_to_grid_indices(Vec2::ZERO),
        (MAP_SIZE / 2, MAP_SIZE / 2)
    );

    assert_eq!(
        map_grid.pos_to_grid_indices(-Vec2::ONE * TILE_SIZE * MAP_SIZE as f32),
        (0, 0)
    );

    assert_eq!(
        map_grid.pos_to_grid_indices(Vec2::ONE * TILE_SIZE * MAP_SIZE as f32),
        (MAP_SIZE - 1, MAP_SIZE - 1)
    );

    assert_eq!(
        map_grid.pos_to_grid_indices(Vec2::ONE * TILE_SIZE),
        (MAP_SIZE / 2 + 1, MAP_SIZE / 2 + 1)
    );

    assert_eq!(
        map_grid.pos_to_grid_indices(
            -Vec2::ONE * 0.5 * TILE_SIZE * MAP_SIZE as f32 + Vec2::ONE * TILE_SIZE
        ),
        (1, 1)
    );
}

#[test]
fn validate_grid_indices_to_pos() {
    let map_grid = MapGrid::empty();

    assert_eq!(
        map_grid.grid_indices_to_pos(0, 0),
        -Vec2::ONE * TILE_SIZE * 0.5 * MAP_SIZE as f32
    );
    assert_eq!(
        map_grid.grid_indices_to_pos(MAP_SIZE / 2, MAP_SIZE / 2),
        Vec2::ZERO
    );
    assert_eq!(
        map_grid.grid_indices_to_pos(MAP_SIZE / 2 + 1, MAP_SIZE / 2),
        Vec2::new(TILE_SIZE, 0.0)
    );
}

#[test]
fn validate_grid_index() {
    let map_grid = MapGrid::empty();

    map_grid.grid_index(0, 0);
    map_grid.grid_index(MAP_SIZE, 0);
    map_grid.grid_index(MAP_SIZE + 100, 0);
    map_grid.grid_index(MAP_SIZE + 100, MAP_SIZE);
    map_grid.grid_index(MAP_SIZE + 100, MAP_SIZE + 100);
}

#[test]
fn validate_grid_position_from_player_pos() {
    let map_grid = MapGrid::empty();

    assert_ne!(
        map_grid.grid_position_from_player_pos(Vec2::ZERO, (2, 1)),
        Vec2::ZERO
    );
}

#[test]
fn validate_random_grid_positions() {
    let map_grid = MapGrid::empty();

    for i in 0..10000 {
        assert_ne!(
            map_grid.random_grid_position_near_player_pos(Vec2::ZERO, (2, 1)),
            Vec2::ZERO,
            "try: {}",
            i
        );
        assert_ne!(
            map_grid.random_grid_position_near_player_pos(Vec2::ZERO, (1, 1)),
            Vec2::ZERO,
            "try: {}",
            i
        );
    }
}
