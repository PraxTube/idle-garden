use bevy::{color::palettes::css::BLUE, prelude::*};

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
                Blueprint { item: ev.flora },
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
            Sprite {
                image: assets.flora_images[ev.flora.index()].clone(),
                color: BLUE.into(),
                ..default()
            },
        ));
    }
}

fn move_blueprint(
    gaming_input: Res<GamingInput>,
    map_data: Res<MapData>,
    mut q_blueprints: Query<&mut Transform, With<Blueprint>>,
) {
    let (x, y) = map_data.pos_to_grid_indices(gaming_input.mouse_world_coords);
    let pos = map_data.grid_indices_to_pos(x, y);

    for mut transform in &mut q_blueprints {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
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

pub struct MapBuildingPlugin;

impl Plugin for MapBuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_building_grid)
            .add_systems(
                Update,
                (
                    spawn_blueprint_item.run_if(
                        resource_exists::<GameAssets>.and(resource_exists::<BachelorBuild>),
                    ),
                    move_blueprint.run_if(resource_exists::<MapData>),
                    display_building_grid,
                    despawn_blueprint.run_if(on_event::<ItemBought>),
                )
                    .chain()
                    .in_set(BuildingSystemSet)
                    .after(ProgressionSystemSet),
            );
    }
}
