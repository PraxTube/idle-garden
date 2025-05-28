use bevy::{color::palettes::css::BLUE, prelude::*};

use crate::{player::GamingInput, world::TILE_SIZE, GameAssets, GameState};

use super::{Flora, ItemBought, MapData, ZLevel};

#[derive(Component)]
struct Blueprint {
    item: Flora,
}
#[derive(Component)]
struct BuildingGrid;

fn spawn_blueprint_item(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_item_bought: EventReader<ItemBought>,
) {
    for ev in ev_item_bought.read() {
        commands.spawn((
            Blueprint { item: ev.item },
            Transform::from_xyz(0.0, 0.0, ZLevel::TopUi.value()),
            Sprite {
                image: assets.flora_images[ev.item.index()].clone(),
                color: BLUE.into(),
                ..default()
            },
        ));
    }
}

fn move_blueprint(
    gaming_input: Res<GamingInput>,
    map_data: Res<MapData>,
    mut q_blueprints: Query<(&mut Transform, &Blueprint)>,
) {
    let (x, y) = map_data.pos_to_grid_indices(gaming_input.mouse_world_coords);
    let pos = map_data.grid_indices_to_pos(x, y);

    for (mut transform, blueprint) in &mut q_blueprints {
        let pos = pos + map_data.flora_data(blueprint.item.index()).gfx_offset();
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

fn spawn_building_grid(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        BuildingGrid,
        Visibility::Hidden,
        Transform::from_xyz(0.0, 0.0, ZLevel::TopEnvironment.value()),
        Sprite::from_image(assets.building_grid.clone()),
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

    grid_transform.translation.x = blueprint_transform.translation.x + TILE_SIZE * 0.5;
    grid_transform.translation.y = blueprint_transform.translation.y;
}

pub struct MapBuildingPlugin;

impl Plugin for MapBuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_building_grid)
            .add_systems(
                Update,
                (
                    spawn_blueprint_item.run_if(resource_exists::<GameAssets>),
                    move_blueprint.run_if(resource_exists::<MapData>),
                    display_building_grid,
                )
                    .chain(),
            );
    }
}
