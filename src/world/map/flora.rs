use bevy::{platform::collections::HashMap, prelude::*};
use serde::Deserialize;
use strum::FromRepr;

use crate::{
    assets::FLORA_DATA_CORE,
    player::Player,
    ui::ItemPressed,
    world::{
        camera::YSort,
        collisions::{StaticCollider, WORLD_COLLISION_GROUPS},
        TILE_SIZE,
    },
    GameAssets,
};

use super::{MapGrid, ProgressionCore, MAP_SIZE};

#[derive(Resource, Deserialize)]
struct FloraDataCore(HashMap<Flora, FloraData>);

#[derive(Deserialize, Clone)]
struct FloraData {
    cost: u32,
    pps: u32,
    ysort: f32,
    gfx_offset: (f32, f32),
    collider_size: (f32, f32),
    size_on_grid: (usize, usize),
}

#[derive(Clone, Copy, Deserialize, Hash, Eq, PartialEq, Default, FromRepr)]
pub enum Flora {
    #[default]
    Potatoe,
    Raddish,
    Carrot,
    Sunflower,
    Tree,
}

impl FloraDataCore {
    fn flora_data(&self, flora: &Flora) -> FloraData {
        self.0
            .get(flora)
            .expect("failed to get flora data from hashmap")
            .clone()
    }
}

impl Flora {
    fn from_index(index: usize) -> Self {
        match Self::from_repr(index) {
            Some(r) => r,
            None => {
                error!("failed to create Flora from index, probably backwards incompatibility.");
                Self::default()
            }
        }
    }

    fn last() -> Self {
        Flora::Tree
    }

    pub fn len() -> usize {
        Flora::last().index() + 1
    }

    pub fn index(&self) -> usize {
        *self as usize
    }

    pub fn image(&self, assets: &GameAssets) -> Handle<Image> {
        assets.flora_images[self.index()].clone()
    }

    pub fn icon(&self, assets: &GameAssets) -> Handle<Image> {
        assets.flora_icons[self.index()].clone()
    }
}

impl FloraData {
    fn ysort(&self) -> YSort {
        YSort(self.ysort)
    }

    fn gfx_offset(&self) -> Vec2 {
        let (x, y) = self.gfx_offset;
        Vec2::new(x, y)
    }

    fn collider(&self) -> StaticCollider {
        let (x, y) = self.collider_size;
        StaticCollider::new(x, y)
    }

    fn size_on_grid(&self) -> (usize, usize) {
        let (x, y) = self.size_on_grid;
        debug_assert!(x > 0);
        debug_assert!(y > 0);
        (x, y)
    }

    fn size_offset(&self) -> Vec2 {
        let (x, y) = self.size_on_grid;

        debug_assert!(x > 0);
        debug_assert!(y > 0);

        0.5 * TILE_SIZE * Vec2::new((x - 1) as f32, (y - 1) as f32)
    }
}

fn spawn_flora(
    commands: &mut Commands,
    assets: &GameAssets,
    pos: Vec2,
    flora: &Flora,
    data: &FloraData,
) {
    let image = flora.image(assets);
    let collider = data.collider();
    let ysort = data.ysort();
    let gfx_offset = data.gfx_offset();

    let root = commands
        .spawn((
            Transform::from_translation(pos.extend(0.0)),
            ysort,
            Visibility::Inherited,
            collider,
            WORLD_COLLISION_GROUPS,
        ))
        .id();

    commands.spawn((
        ChildOf(root),
        Transform::from_translation(gfx_offset.extend(0.0)),
        Sprite::from_image(image),
    ));
}

fn spawn_flora_from_item_pressed(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut map_grid: ResMut<MapGrid>,
    core: Res<FloraDataCore>,
    q_player: Query<&Transform, With<Player>>,
    mut ev_item_pressed: EventReader<ItemPressed>,
) {
    let Ok(transform) = q_player.single() else {
        debug_assert!(ev_item_pressed.is_empty());
        return;
    };

    for ev in ev_item_pressed.read() {
        let flora_data = core.flora_data(&ev.flora);
        let pos = map_grid.random_grid_position_near_player_pos(
            transform.translation.xy(),
            flora_data.size_on_grid(),
        );

        map_grid.set_map_grid_value_at_pos(pos, flora_data.size_on_grid(), ev.flora.index() as u16);
        spawn_flora(
            &mut commands,
            &assets,
            pos + flora_data.size_offset(),
            &ev.flora,
            &flora_data,
        );
    }
}

fn increment_progression_core_flora(
    mut core: ResMut<ProgressionCore>,
    mut ev_item_pressed: EventReader<ItemPressed>,
) {
    for ev in ev_item_pressed.read() {
        core.flora[ev.flora.index()] += 1;
    }
}

fn insert_flora_data_core(mut commands: Commands) {
    let core: FloraDataCore =
        serde_json::from_str(FLORA_DATA_CORE).expect("failed to parse flora data core to json str");
    commands.insert_resource(core);
}

fn spawn_flora_on_map_grid_insertion(
    mut commands: Commands,
    assets: Res<GameAssets>,
    core: Res<FloraDataCore>,
    map_grid: Res<MapGrid>,
) {
    info!("spawn");
    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            if map_grid.grid[x][y] == u16::MAX {
                continue;
            }

            let pos = map_grid.grid_indices_to_pos(x, y);

            let flora = Flora::from_index(map_grid.grid_index(x, y).into());
            let flora_data = core.flora_data(&flora);

            spawn_flora(&mut commands, &assets, pos, &flora, &flora_data);
        }
    }
}

pub struct MapFloraPlugin;

impl Plugin for MapFloraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_flora_from_item_pressed
                    .run_if(resource_exists::<GameAssets>.and(resource_exists::<MapGrid>)),
                increment_progression_core_flora.run_if(resource_exists::<ProgressionCore>),
            ),
        )
        .add_systems(Startup, insert_flora_data_core)
        .add_systems(
            Update,
            spawn_flora_on_map_grid_insertion.run_if(
                resource_exists::<GameAssets>
                    .and(resource_exists::<MapGrid>)
                    .and(resource_exists::<FloraDataCore>)
                    .and(run_once),
            ),
        );
    }
}
