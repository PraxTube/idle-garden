use bevy::{prelude::*, text::FontSmoothing};

use crate::{player::Player, world::DebugState, GameAssets, GameState};

use super::outline::TextOutline;

#[derive(Component)]
struct DebugStateText;
#[derive(Component)]
struct PlayerTransformText;

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

fn spawn_player_transform_text(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        PlayerTransformText,
        Visibility::Hidden,
        Node {
            top: Val::Px(250.0),
            left: Val::Percent(50.0),
            ..default()
        },
        TextOutline::new(
            String::new(),
            1.0,
            Color::WHITE,
            Color::BLACK,
            TextFont {
                font: assets.pixel_font.clone(),
                font_size: 25.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            true,
        ),
    ));
}

fn toggle_player_transform_text_visibility(
    debug_state: Res<DebugState>,
    mut q_visibility: Query<&mut Visibility, With<PlayerTransformText>>,
) {
    let Ok(mut visiblity) = q_visibility.single_mut() else {
        return;
    };

    *visiblity = if debug_state.active && debug_state.player_transform_debug_active {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
}

fn update_player_transform_text(
    q_player: Query<&Transform, With<Player>>,
    mut q_outline: Query<&mut TextOutline, With<PlayerTransformText>>,
) {
    let Ok(transform) = q_player.single() else {
        return;
    };

    let Ok(mut outline) = q_outline.single_mut() else {
        return;
    };

    let string = format!(
        "x: {:.1}, y: {:.1}, z: {:.1}",
        transform.translation.x, transform.translation.y, transform.translation.z
    );

    outline.text = string;
}

pub struct UiDebugPlugin;

impl Plugin for UiDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_player_transform_text)
            .add_systems(
                Update,
                (spawn_debug_state_text, despawn_debug_state_text)
                    .chain()
                    .run_if(resource_exists::<GameAssets>),
            )
            .add_systems(
                Update,
                (
                    update_player_transform_text,
                    toggle_player_transform_text_visibility,
                )
                    .chain(),
            );
    }
}
