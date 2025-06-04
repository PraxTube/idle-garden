use bevy::prelude::*;
use serde::Deserialize;
use strum::FromRepr;

use crate::{
    world::{
        camera::YSort,
        collisions::{IntersectionEvent, StaticCollider, WORLD_COLLISION_GROUPS},
        TILE_SIZE,
    },
    BachelorBuild, GameAssets,
};

use super::{
    ItemBought, MapData, ProgressionCore, EMPTY_CELL_VALUE, MAP_SIZE, PLAYER_BLOCKED_CELL_VALUE,
};

#[derive(Deserialize, Clone, Default)]
pub struct FloraData {
    pub cost: u32,
    pub pps: u32,
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

impl Flora {
    pub fn from_index(index: usize) -> Self {
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

    pub fn gfx_offset(&self) -> Vec2 {
        let (x, y) = self.gfx_offset;
        Vec2::new(x, y)
    }

    fn collider(&self) -> StaticCollider {
        let (x, y) = self.collider_size;
        StaticCollider::new(x, y)
    }

    pub fn size_on_grid(&self) -> (usize, usize) {
        let (x, y) = self.size_on_grid;
        debug_assert!(x > 0);
        debug_assert!(y > 0);
        (x, y)
    }

    /// Offset that will center the item based on its grid size.
    /// Has nothing to do with the `gfx_offset`, they are two separate things.
    /// Altough they are often very similar.
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

fn spawn_flora_on_item_bought(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut map_data: ResMut<MapData>,
    bachelor_build: Res<BachelorBuild>,
    mut ev_item_bought: EventReader<ItemBought>,
) {
    if !bachelor_build.with_building {
        return;
    }

    for ev in ev_item_bought.read() {
        let flora_data = map_data.flora_data(ev.item.index());
        let pos = ev.pos;

        map_data.set_map_data_value_at_pos(pos, flora_data.size_on_grid(), ev.item.index() as u16);
        spawn_flora(
            &mut commands,
            &assets,
            pos + flora_data.size_offset(),
            &ev.item,
            &flora_data,
        );
    }
}

fn spawn_flora_on_map_data_insertion(
    mut commands: Commands,
    assets: Res<GameAssets>,
    map_data: Res<MapData>,
) {
    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            if map_data.grid[x][y] == EMPTY_CELL_VALUE
                || map_data.grid[x][y] == PLAYER_BLOCKED_CELL_VALUE
            {
                continue;
            }

            let pos = map_data.grid_indices_to_pos(x, y);

            let flora = Flora::from_index(map_data.grid_index(x, y).into());
            let flora_data = map_data.flora_data(flora.index());

            spawn_flora(&mut commands, &assets, pos, &flora, &flora_data);
        }
    }
}

fn cut_tall_grass(
    mut commands: Commands,
    mut core: ResMut<ProgressionCore>,
    mut ev_intersection: EventReader<IntersectionEvent>,
) {
    for ev in ev_intersection.read() {
        core.points += 1;
        commands.entity(ev.aabb).despawn();
    }
}

pub struct MapFloraPlugin;

impl Plugin for MapFloraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_flora_on_item_bought.run_if(
                    resource_exists::<GameAssets>
                        .and(resource_exists::<MapData>.and(resource_exists::<BachelorBuild>)),
                ),
                spawn_flora_on_map_data_insertion.run_if(
                    resource_exists::<GameAssets>
                        .and(resource_exists::<MapData>)
                        .and(run_once),
                ),
                cut_tall_grass.run_if(resource_exists::<ProgressionCore>),
            ),
        );
    }
}
