use bevy::{prelude::*, text::FontSmoothing};

use crate::{world::ProgressionCore, GameAssets, GameState};

use super::outline::TextOutline;

#[derive(Component)]
struct StatsRoot;
#[derive(Component)]
struct PointsText;
#[derive(Component)]
struct PointsCapText;
#[derive(Component)]
struct PointsPerSecondText;

fn spawn_stats(mut commands: Commands, assets: Res<GameAssets>) {
    let root = commands
        .spawn((
            StatsRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    let container = commands
        .spawn((
            ChildOf(root),
            Node {
                width: Val::Percent(90.0),
                height: Val::Percent(100.0),
                top: Val::Px(50.0),
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    commands.spawn((
        ChildOf(container),
        PointsText,
        Node {
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            ..default()
        },
        TextOutline::new(
            "$313101".to_string(),
            1.0,
            Color::WHITE,
            Color::BLACK,
            TextFont {
                font: assets.pixel_font.clone(),
                font_size: 35.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            false,
        ),
    ));

    commands.spawn((
        ChildOf(container),
        PointsCapText,
        Node {
            left: Val::Px(0.0),
            top: Val::Px(120.0),
            ..default()
        },
        TextOutline::new(
            "$313101".to_string(),
            1.0,
            Color::WHITE,
            Color::BLACK,
            TextFont {
                font: assets.pixel_font.clone(),
                font_size: 35.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            false,
        ),
    ));

    commands.spawn((
        ChildOf(container),
        PointsPerSecondText,
        Node {
            left: Val::Px(0.0),
            top: Val::Px(60.0),
            width: Val::Percent(100.0),
            ..default()
        },
        TextOutline::new(
            "$313101/s".to_string(),
            1.0,
            Color::WHITE,
            Color::BLACK,
            TextFont {
                font: assets.pixel_font.clone(),
                font_size: 35.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            false,
        ),
    ));
}

fn update_points_text(
    core: Res<ProgressionCore>,
    mut q_text: Query<&mut TextOutline, With<PointsText>>,
) {
    let Ok(mut outline) = q_text.single_mut() else {
        return;
    };

    outline.text = format!("${}", core.points)
}

fn update_points_cap_text(
    core: Res<ProgressionCore>,
    mut q_text: Query<&mut TextOutline, With<PointsCapText>>,
) {
    let Ok(mut outline) = q_text.single_mut() else {
        return;
    };

    outline.text = format!("Cap ${}", core.points_cap)
}

fn update_points_per_second_text(
    core: Res<ProgressionCore>,
    mut q_text: Query<&mut TextOutline, With<PointsPerSecondText>>,
) {
    let Ok(mut outline) = q_text.single_mut() else {
        return;
    };

    outline.text = format!("+{}/s", core.pps)
}

pub struct UiStatsPlugin;

impl Plugin for UiStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_stats)
            .add_systems(
                Update,
                (
                    update_points_text,
                    update_points_cap_text,
                    update_points_per_second_text,
                )
                    .run_if(resource_exists::<ProgressionCore>),
            );
    }
}
