use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin},
};

use serde::Deserialize;
use strum::FromRepr;

use crate::{
    assets::FLORA_SHADER,
    world::{
        camera::YSort,
        collisions::{StaticCollider, WORLD_COLLISION_GROUPS},
        TILE_SIZE,
    },
    BachelorBuild, GameAssets,
};

use super::{ItemBought, MapData, EMPTY_CELL_VALUE, MAP_SIZE, PLAYER_BLOCKED_CELL_VALUE};

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
    SwampTree,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FloraMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Option<Handle<Image>>,
    #[texture(2)]
    #[sampler(3)]
    pub noise_texture: Option<Handle<Image>>,
    #[uniform(4)]
    pub texel_size: Vec2,
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
        Flora::SwampTree
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

impl Material2d for FloraMaterial {
    fn fragment_shader() -> ShaderRef {
        FLORA_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

fn spawn_flora(
    commands: &mut Commands,
    assets: &GameAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<FloraMaterial>,
    images: &Assets<Image>,
    pos: Vec2,
    flora: &Flora,
    data: &FloraData,
) {
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

    let image_handle = flora.image(assets);
    let Some(image) = images.get(&image_handle) else {
        error!("failed to get image from image handle when spawning flora, must never happen!");
        return;
    };

    commands.spawn((
        ChildOf(root),
        Transform::from_translation(gfx_offset.extend(0.0)).with_scale(Vec3::new(
            image.width() as f32,
            image.height() as f32,
            1.0,
        )),
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(
            materials
                .add(FloraMaterial {
                    texture: Some(image_handle.clone()),
                    noise_texture: Some(assets.noise_texture.clone()),
                    texel_size: Vec2::new(1.0 / image.width() as f32, 1.0 / image.height() as f32),
                })
                .clone(),
        ),
    ));
}

fn spawn_flora_on_item_bought(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FloraMaterial>>,
    images: Res<Assets<Image>>,
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
            &mut meshes,
            &mut materials,
            &images,
            pos,
            &ev.item,
            &flora_data,
        );
    }
}

fn spawn_flora_on_map_data_insertion(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FloraMaterial>>,
    images: Res<Assets<Image>>,
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

            spawn_flora(
                &mut commands,
                &assets,
                &mut meshes,
                &mut materials,
                &images,
                pos,
                &flora,
                &flora_data,
            );
        }
    }
}

pub struct MapFloraPlugin;

impl Plugin for MapFloraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<FloraMaterial>::default())
            .add_systems(
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
                ),
            );
    }
}
