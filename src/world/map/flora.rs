use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin},
};

use serde::Deserialize;
use strum::FromRepr;

use crate::{
    assets::FLORA_SHADER,
    ui::{MenuAction, MenuActionEvent},
    world::{
        camera::YSort,
        collisions::{StaticCollider, WORLD_COLLISION_GROUPS},
        TILE_SIZE,
    },
    BachelorBuild, EffectAssets, GameAssets,
};

use super::{ItemBought, MapData, EMPTY_CELL_VALUE, MAP_SIZE, TALL_GRASS_CELL_VALUE};

#[derive(Deserialize, Clone, Default)]
pub struct FloraData {
    base_cost: u32,
    pub cost_growth_factor: f32,
    // If false, will use linear growth.
    exponential_growth: bool,
    pub pps: u32,
    tree_shader: bool,
    ysort: f32,
    gfx_offset: (f32, f32),
    collider_size: (f32, f32),
    size_on_grid: (usize, usize),
}

#[derive(Clone, Copy, Deserialize, Hash, Eq, PartialEq, Default, FromRepr, Debug)]
pub enum Flora {
    #[default]
    Potatoe,
    Raddish,
    Carrot,
    Sunflower,
    Tree,
    SwampTree,
}

/// This is used as an Event, but because Events are a little more boiler plate I opted to use just
/// a resource. We insert this when we spawn the flora, only after that do we want to spawn the
/// grass (and only at places where there is no flora already).
#[derive(Resource)]
pub struct InitialFloraSpawned;

#[derive(Component)]
struct FloraMarker;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FloraMaterial {
    #[uniform(0)]
    pub texel_size: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
}

impl Flora {
    pub fn from_index(index: usize) -> Option<Self> {
        let maybe_flora = Self::from_repr(index);

        if maybe_flora.is_none() {
            error!("failed to create Flora from index, probably backwards incompatibility.");
        }
        maybe_flora
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

    pub fn cost(&self, count: usize) -> u32 {
        if self.exponential_growth {
            self.base_cost * (self.cost_growth_factor.powi(count as i32)).floor() as u32
        } else {
            self.base_cost + (self.cost_growth_factor).round() as u32 * count as u32
        }
    }

    fn collider(&self) -> Option<StaticCollider> {
        let (x, y) = self.collider_size;
        if x == 0.0 && y == 0.0 {
            return None;
        }
        Some(StaticCollider::new(x, y))
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
    #[allow(dead_code)]
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
    effects: &EffectAssets,
    materials: &mut Assets<FloraMaterial>,
    images: &Assets<Image>,
    pos: Vec2,
    flora: &Flora,
    data: &FloraData,
) {
    let maybe_collider = data.collider();
    let ysort = data.ysort();
    let gfx_offset = data.gfx_offset();

    let root = commands
        .spawn((
            FloraMarker,
            Transform::from_translation(pos.extend(0.0)),
            ysort,
            Visibility::Inherited,
            WORLD_COLLISION_GROUPS,
        ))
        .id();

    if let Some(collider) = maybe_collider {
        commands.entity(root).insert(collider);
    }

    let image_handle = flora.image(assets);
    let Some(image) = images.get(&image_handle) else {
        error!("failed to get image from image handle when spawning flora, must never happen!");
        return;
    };

    let image_size = Vec2::new(image.width() as f32, image.height() as f32);

    let entity = commands.spawn(ChildOf(root)).id();

    if data.tree_shader {
        commands
            .entity(entity)
            .insert(
                Transform::from_translation(gfx_offset.extend(0.0))
                    .with_scale(image_size.extend(1.0)),
            )
            .insert(Mesh2d(effects.rect_mesh.clone()))
            .insert(MeshMaterial2d(
                materials
                    .add(FloraMaterial {
                        texel_size: (1.0 / image_size).extend(0.0).extend(0.0),
                        texture: Some(image_handle.clone()),
                    })
                    .clone(),
            ));
    } else {
        commands
            .entity(entity)
            .insert(Transform::from_translation(gfx_offset.extend(0.0)))
            .insert(Sprite {
                image: image_handle.clone(),
                ..default()
            });
    }
}

fn spawn_flora_on_item_bought(
    mut commands: Commands,
    assets: Res<GameAssets>,
    effects: Res<EffectAssets>,
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
            &effects,
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
    effects: Res<EffectAssets>,
    mut materials: ResMut<Assets<FloraMaterial>>,
    images: Res<Assets<Image>>,
    map_data: Res<MapData>,
) {
    commands.insert_resource(InitialFloraSpawned);
    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            if map_data.grid_index(x, y) == EMPTY_CELL_VALUE
                || map_data.grid_index(x, y) == TALL_GRASS_CELL_VALUE
            {
                continue;
            }

            let pos = map_data.grid_indices_to_pos(x, y);

            let Some(flora) = Flora::from_index(map_data.grid_index(x, y).into()) else {
                continue;
            };
            let flora_data = map_data.flora_data(flora.index());

            spawn_flora(
                &mut commands,
                &assets,
                &effects,
                &mut materials,
                &images,
                pos,
                &flora,
                &flora_data,
            );
        }
    }
}

fn despawn_flora_on_reset(
    mut commands: Commands,
    q_floras: Query<Entity, With<FloraMarker>>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    if !ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::Reset)
    {
        return;
    }

    for entity in &q_floras {
        commands.entity(entity).despawn();
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
                            .and(resource_exists::<EffectAssets>)
                            .and(resource_exists::<MapData>)
                            .and(run_once),
                    ),
                    despawn_flora_on_reset,
                ),
            );
    }
}
