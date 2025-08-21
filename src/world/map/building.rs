use bevy::{color::palettes::css::RED, prelude::*};
use bevy_trickfilm::prelude::*;

use crate::{player::GamingInput, ui::ItemPressed, BachelorBuild, GameAssets};

use super::{Flora, MapData, ProgressionCore, ProgressionSystemSet, ZLevel};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuildingSystemSet;

#[derive(Component)]
pub struct Blueprint {
    pub item: Flora,
    pub fits_at_pos: bool,
}

fn spawn_blueprint_item(
    mut commands: Commands,
    assets: Res<GameAssets>,
    core: Res<ProgressionCore>,
    map_data: Res<MapData>,
    bachelor_build: Res<BachelorBuild>,
    mut ev_item_pressed: EventReader<ItemPressed>,
) {
    if !bachelor_build.with_building {
        return;
    }

    for ev in ev_item_pressed.read() {
        if !core.is_affordable(&map_data, &ev.flora) {
            continue;
        }

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

        let mut animator = AnimationPlayer2D::default();
        animator
            .play(assets.building_selector_animation.clone())
            .repeat();

        commands.spawn((
            ChildOf(root),
            animator,
            Sprite::from_atlas_image(
                assets.building_selector.clone(),
                assets.building_selector_layout.clone().into(),
            ),
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

fn despawn_blueprint(mut commands: Commands, q_blueprint: Query<Entity, With<Blueprint>>) {
    for entity in &q_blueprint {
        commands.entity(entity).despawn();
    }
}

fn despawn_blueprint_on_player_input(
    mut commands: Commands,
    gaming_input: Res<GamingInput>,
    q_blueprint: Query<Entity, With<Blueprint>>,
) {
    if !gaming_input.cancel {
        return;
    }

    for entity in &q_blueprint {
        commands.entity(entity).despawn();
    }
}

fn despawn_blueprint_if_not_affordable(
    mut commands: Commands,
    core: Res<ProgressionCore>,
    map_data: Res<MapData>,
    q_blueprints: Query<(Entity, &Blueprint)>,
) {
    debug_assert!(q_blueprints.iter().count() <= 1);

    for (entity, blueprint) in &q_blueprints {
        if !core.is_affordable(&map_data, &blueprint.item) {
            commands.entity(entity).despawn();
        }
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
            Color::WHITE
        } else {
            RED.into()
        };

        sprite.color = color;
    }
}

pub struct MapBuildingPlugin;

impl Plugin for MapBuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                despawn_blueprint.run_if(on_event::<ItemPressed>),
                despawn_blueprint_on_player_input,
                spawn_blueprint_item
                    .run_if(resource_exists::<GameAssets>.and(resource_exists::<BachelorBuild>)),
                move_blueprint.run_if(resource_exists::<MapData>),
                despawn_blueprint_if_not_affordable
                    .run_if(resource_exists::<ProgressionCore>.and(resource_exists::<MapData>)),
                update_blueprint_color,
            )
                .chain()
                .in_set(BuildingSystemSet)
                .after(ProgressionSystemSet),
        );
    }
}
