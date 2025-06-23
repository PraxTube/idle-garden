use bevy::{prelude::*, text::FontSmoothing};

use crate::{world::DebugState, GameAssets};

use super::outline::TextOutline;

#[derive(Component)]
struct DebugStateText;

fn spawn_debug_state_text(
    mut commands: Commands,
    assets: Res<GameAssets>,
    debug_state: Res<DebugState>,
) {
    if !debug_state.changed_this_frame || !debug_state.active {
        return;
    }

    commands.spawn((
        DebugStateText,
        Node {
            left: Val::Percent(50.0),
            top: Val::Px(50.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        TextOutline::new(
            "DEBUG MODE".to_string(),
            1.0,
            Color::WHITE.with_alpha(0.75),
            Color::BLACK.with_alpha(0.75),
            TextFont {
                font: assets.pixel_font.clone(),
                font_size: 40.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            true,
        ),
    ));
}

fn despawn_debug_state_text(
    mut commands: Commands,
    debug_state: Res<DebugState>,
    q_debug_state_text: Query<Entity, With<DebugStateText>>,
) {
    if !debug_state.changed_this_frame || debug_state.active {
        return;
    }

    for entity in q_debug_state_text {
        commands.entity(entity).despawn();
    }
}

pub struct UiDebugPlugin;

impl Plugin for UiDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_debug_state_text, despawn_debug_state_text)
                .chain()
                .run_if(resource_exists::<GameAssets>),
        );
    }
}
