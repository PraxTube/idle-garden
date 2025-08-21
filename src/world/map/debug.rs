use bevy::{prelude::*, text::FontSmoothing};

use crate::{
    player::Player,
    world::{DebugState, TILE_SIZE},
    GameAssets,
};

use super::{MapData, ZLevel, EMPTY_CELL_VALUE, TALL_GRASS_CELL_VALUE};

const DEBUG_GRID_SIZE: usize = 50;

#[derive(Component)]
struct GridDebugVisual;

fn spawn_grid_debug_visuals(
    mut commands: Commands,
    assets: Res<GameAssets>,
    debug_state: Res<DebugState>,
    map_data: Res<MapData>,
    q_player: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = q_player.single() else {
        return;
    };

    if !debug_state.active || !debug_state.grid_debug_active {
        return;
    }

    let (player_x, player_y) = map_data.pos_to_grid_indices(player_transform.translation.xy());

    for i in 0..DEBUG_GRID_SIZE {
        for j in 0..DEBUG_GRID_SIZE {
            let x = i + player_x.max(DEBUG_GRID_SIZE / 2) - DEBUG_GRID_SIZE / 2;
            let y = j + player_y.max(DEBUG_GRID_SIZE / 2) - DEBUG_GRID_SIZE / 2;

            if !map_data.indices_in_grid(x, y) {
                continue;
            }

            let index = map_data.grid_index(x, y);
            let (text, color) = if index == EMPTY_CELL_VALUE {
                ("E".to_string(), Color::WHITE.with_alpha(0.5))
            } else if index == TALL_GRASS_CELL_VALUE {
                ("G".to_string(), Color::WHITE.with_alpha(0.75))
            } else {
                (format!("{:X}", index), Color::WHITE)
            };

            let pos = map_data.grid_indices_to_pos(x, y);
            commands.spawn((
                GridDebugVisual,
                Text2d::new(text),
                TextFont {
                    font: assets.pixel_font.clone(),
                    font_size: 120.0,
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                TextColor(color),
                Transform::from_xyz(pos.x, pos.y, ZLevel::TopUi.value())
                    .with_scale(Vec3::splat(0.1)),
            ));

            commands.spawn((
                GridDebugVisual,
                Sprite {
                    color: Color::BLACK.with_alpha(0.5),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, ZLevel::TopUi.value() - 10.0),
            ));
        }
    }
}

fn despawn_grid_debug_visuals(
    mut commands: Commands,
    q_grid_debug_visuals: Query<Entity, With<GridDebugVisual>>,
) {
    for entity in &q_grid_debug_visuals {
        commands.entity(entity).despawn();
    }
}

pub struct MapDebugPlugin;

impl Plugin for MapDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                despawn_grid_debug_visuals,
                spawn_grid_debug_visuals.run_if(resource_exists::<GameAssets>),
            )
                .chain()
                .run_if(resource_exists::<MapData>),
        );
    }
}

#[test]
fn validate_debug_grid_size_is_halvable() {
    assert!(DEBUG_GRID_SIZE % 2 == 0)
}
