use bevy::{
    color::palettes::css::{BLUE, RED},
    prelude::*,
};

use crate::{
    player::GamingInput, ui::ItemPressed, world::TILE_SIZE, BachelorBuild, GameAssets, GameState,
};

use super::{Flora, ItemBought, MapData, ProgressionSystemSet, ZLevel};

const USER_GRID_OFFSET: Vec2 = Vec2::new(0.5 * TILE_SIZE, 0.5 * TILE_SIZE);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuildingSystemSet;

#[derive(Component)]
pub struct Blueprint {
    pub item: Flora,
    pub fits_at_pos: bool,
}
#[derive(Component)]
struct BuildingGrid;

fn spawn_blueprint_item(
    mut commands: Commands,
    assets: Res<GameAssets>,
    map_data: Res<MapData>,
    bachelor_build: Res<BachelorBuild>,
    mut ev_item_pressed: EventReader<ItemPressed>,
) {
    if !bachelor_build.with_building {
        return;
    }

    for ev in ev_item_pressed.read() {
        let root = commands
            .spawn((
                Blueprint {
                    item: ev.flora,
                    fits_at_pos: false,
                },
                Visibility::Inherited,
                Transform::from_xyz(0.0, 0.0, ZLevel::TopUi.value()),
            ))
            .id();

        commands.spawn((
            ChildOf(root),
            Transform::from_translation(
                map_data
                    .flora_data(ev.flora.index())
                    .gfx_offset()
                    .extend(0.0),
            ),
            Sprite::from_image(assets.flora_images[ev.flora.index()].clone()),
        ));
    }
}

fn move_blueprint(
    gaming_input: Res<GamingInput>,
    map_data: Res<MapData>,
    mut q_blueprint: Query<(&mut Transform, &mut Blueprint)>,
) {
    let (x, y) = map_data.pos_to_grid_indices(gaming_input.mouse_world_coords);
    let pos = map_data.grid_indices_to_pos(x, y);

    let Ok((mut transform, mut blueprint)) = q_blueprint.single_mut() else {
        return;
    };

    transform.translation.x = pos.x;
    transform.translation.y = pos.y;

    blueprint.fits_at_pos = map_data.fits_at_pos(
        transform.translation.xy(),
        map_data.flora_data(blueprint.item.index()).size_on_grid(),
    );
}

fn spawn_building_grid(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        BuildingGrid,
        Visibility::Hidden,
        Transform::from_xyz(0.0, 0.0, ZLevel::TopEnvironment.value()),
        Sprite {
            image: assets.building_grid.clone(),
            color: Color::WHITE.with_alpha(0.5),
            ..default()
        },
    ));
}

fn display_building_grid(
    q_blueprint: Query<&Transform, With<Blueprint>>,
    mut q_building_grid: Query<
        (&mut Transform, &mut Visibility),
        (With<BuildingGrid>, Without<Blueprint>),
    >,
) {
    let Ok((mut grid_transform, mut visibility)) = q_building_grid.single_mut() else {
        return;
    };

    *visibility = Visibility::Hidden;

    let Ok(blueprint_transform) = q_blueprint.single() else {
        return;
    };

    *visibility = Visibility::Inherited;

    grid_transform.translation.x = blueprint_transform.translation.x + USER_GRID_OFFSET.x;
    grid_transform.translation.y = blueprint_transform.translation.y + USER_GRID_OFFSET.y;
}

fn despawn_blueprint(mut commands: Commands, q_blueprint: Query<Entity, With<Blueprint>>) {
    for entity in &q_blueprint {
        commands.entity(entity).despawn();
    }
}

fn update_blueprint_color(
    q_blueprint: Query<(&Children, &Blueprint)>,
    mut q_sprites: Query<&mut Sprite>,
) {
    let Ok((children, blueprint)) = q_blueprint.single() else {
        return;
    };

    for child in children {
        let Ok(mut sprite) = q_sprites.get_mut(child.entity()) else {
            continue;
        };

        let color = if blueprint.fits_at_pos {
            BLUE.into()
        } else {
            RED.into()
        };

        sprite.color = color;
    }
}

pub struct MapBuildingPlugin;

impl Plugin for MapBuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_building_grid)
            .add_systems(
                Update,
                (
                    despawn_blueprint.run_if(on_event::<ItemPressed>),
                    spawn_blueprint_item.run_if(
                        resource_exists::<GameAssets>.and(resource_exists::<BachelorBuild>),
                    ),
                    move_blueprint.run_if(resource_exists::<MapData>),
                    display_building_grid,
                    despawn_blueprint.run_if(on_event::<ItemBought>),
                    update_blueprint_color,
                )
                    .chain()
                    .in_set(BuildingSystemSet)
                    .after(ProgressionSystemSet),
            );
    }
}
