use bevy::{platform::collections::HashMap, prelude::*};
use serde::Deserialize;

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

use super::{MapGrid, ProgressionCore};

#[derive(Resource, Deserialize)]
struct FloraDataCore(HashMap<Flora, FloraData>);

#[derive(Deserialize)]
struct FloraData {
    cost: u32,
    pps: u32,
    ysort: f32,
    gfx_offset: (f32, f32),
    collider_size: (f32, f32),
    size_on_grid: (usize, usize),
}

#[derive(Clone, Copy, Deserialize, Hash, Eq, PartialEq, Default)]
pub enum Flora {
    #[default]
    Potatoe,
    Raddish,
    Carrot,
    Sunflower,
    Tree,
}

impl FloraDataCore {
    fn ysort(&self, flora: Flora) -> YSort {
        let y = self
            .0
            .get(&flora)
            .expect("failed to get hashmap value from flora")
            .ysort;
        YSort(y)
    }

    fn gfx_offset(&self, flora: Flora) -> Vec2 {
        let (x, y) = self
            .0
            .get(&flora)
            .expect("failed to get hashmap value from flora")
            .gfx_offset;
        Vec2::new(x, y)
    }

    fn collider(&self, flora: Flora) -> StaticCollider {
        let (x, y) = self
            .0
            .get(&flora)
            .expect("failed to get hashmap value from flora")
            .collider_size;
        StaticCollider::new(x, y)
    }

    fn size_on_grid(&self, flora: Flora) -> (usize, usize) {
        let (x, y) = self
            .0
            .get(&flora)
            .expect("failed to get hashmap value from flora")
            .size_on_grid;
        debug_assert!(x > 0);
        debug_assert!(y > 0);
        (x, y)
    }

    fn size_offset(&self, flora: Flora) -> Vec2 {
        let (x, y) = self.size_on_grid(flora);

        debug_assert!(x > 0);
        debug_assert!(y > 0);

        0.5 * TILE_SIZE * Vec2::new((x - 1) as f32, (y - 1) as f32)
    }
}

impl Flora {
    fn last() -> Flora {
        Flora::Tree
    }

    pub fn len() -> usize {
        Flora::last().index() + 1
    }

    pub fn index(&self) -> usize {
        *self as usize
    }

    fn grid_value(&self) -> u16 {
        *self as u16 + 1
    }

    pub fn image(&self, assets: &GameAssets) -> Handle<Image> {
        assets.flora_images[self.index()].clone()
    }

    pub fn icon(&self, assets: &GameAssets) -> Handle<Image> {
        assets.flora_icons[self.index()].clone()
    }
}

fn spawn_flora(
    commands: &mut Commands,
    assets: &GameAssets,
    core: &FloraDataCore,
    pos: Vec2,
    flora: Flora,
) {
    let image = flora.image(assets);
    let collider = core.collider(flora);
    let ysort = core.ysort(flora);
    let gfx_offset = core.gfx_offset(flora);

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
        let pos = map_grid.random_grid_position_near_player_pos(
            transform.translation.xy(),
            core.size_on_grid(ev.flora),
        );

        map_grid.set_map_grid_value_at_pos(pos, core.size_on_grid(ev.flora), ev.flora.grid_value());
        spawn_flora(
            &mut commands,
            &assets,
            &core,
            pos + core.size_offset(ev.flora),
            ev.flora,
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
        .add_systems(Startup, insert_flora_data_core);
    }
}
