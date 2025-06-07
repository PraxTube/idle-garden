use bevy::{
    color::palettes::css::{PURPLE, WHITE},
    prelude::*,
    text::FontSmoothing,
};

use crate::{
    player::{GamingInput, Player},
    world::{DebugState, TILE_SIZE},
    GameAssets,
};

use super::{MapData, ZLevel, EMPTY_CELL_VALUE};

const DEBUG_GRID_SIZE: usize = 50;

#[derive(Component)]
struct GridDebugVisual;

fn toggle_grid_debug(mut debug_state: ResMut<DebugState>, gaming_input: Res<GamingInput>) {
    if !debug_state.active {
        return;
    }
    if !gaming_input.toggle_debug_grid {
        return;
    }

    debug_state.grid_debug_active = !debug_state.grid_debug_active;
}

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

            if map_data.grid_index(x, y) == EMPTY_CELL_VALUE {
                continue;
            }

            let pos = map_data.grid_indices_to_pos(x, y);

            let mut text = format!("{:X}", map_data.grid_index(x, y));
            if text.chars().count() > 2 {
                text.insert(2, '\n');
            }

            commands.spawn((
                GridDebugVisual,
                Text2d::new(text),
                TextFont {
                    font: assets.pixel_font.clone(),
                    font_size: 6.0,
                    font_smoothing: FontSmoothing::None,
                    ..default()
                },
                TextColor(WHITE.into()),
                Transform::from_xyz(pos.x, pos.y, ZLevel::TopUi.value()),
            ));

            commands.spawn((
                GridDebugVisual,
                Sprite {
                    color: PURPLE.with_alpha(0.35).into(),
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
                toggle_grid_debug,
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
