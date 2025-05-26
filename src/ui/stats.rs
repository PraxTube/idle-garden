use bevy::{prelude::*, text::FontSmoothing};

use crate::{world::ProgressionCore, GameAssets, GameState};

const PADDING: f32 = 50.0;
const HEIGHT: f32 = 100.0;
const WIDTH: f32 = 300.0;
const STATS_PADDING_TOP: f32 = 10.0;
const STATS_PADDING_LEFT: f32 = 10.0;

#[derive(Component)]
struct StatsRoot;
#[derive(Component)]
struct StatsBackground;
#[derive(Component)]
struct StatsContainer;
#[derive(Component)]
struct PointsText;
#[derive(Component)]
struct PointsPerSecondText;

fn spawn_stats(mut commands: Commands, assets: Res<GameAssets>) {
    let root = commands
        .spawn((
            StatsRoot,
            Node {
                left: Val::Px(PADDING),
                top: Val::Px(PADDING),
                height: Val::Px(HEIGHT),
                width: Val::Px(WIDTH),
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    commands.spawn((
        ChildOf(root),
        StatsBackground,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        ZIndex(-1),
        ImageNode {
            color: Color::WHITE,
            image: Handle::<Image>::default(),
            ..default()
        },
    ));

    let container = commands
        .spawn((
            ChildOf(root),
            StatsContainer,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect {
                    top: Val::Px(STATS_PADDING_TOP),
                    left: Val::Px(STATS_PADDING_LEFT),
                    ..default()
                },
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    commands.spawn((
        ChildOf(container),
        PointsText,
        Text("Points: 313012".to_string()),
        TextColor(Color::BLACK),
        TextFont {
            font: assets.pixel_font.clone(),
            font_size: 15.0,
            font_smoothing: FontSmoothing::None,
            ..default()
        },
    ));

    commands.spawn((
        ChildOf(container),
        PointsPerSecondText,
        Text("Points/s: 3123".to_string()),
        TextColor(Color::BLACK),
        TextFont {
            font: assets.pixel_font.clone(),
            font_size: 15.0,
            font_smoothing: FontSmoothing::None,
            ..default()
        },
    ));
}

fn update_points_text(core: Res<ProgressionCore>, mut q_text: Query<&mut Text, With<PointsText>>) {
    let Ok(mut text) = q_text.single_mut() else {
        return;
    };

    text.0 = format!("Points: {:>8}", core.points)
}

fn update_points_per_second_text(
    core: Res<ProgressionCore>,
    mut q_text: Query<&mut Text, With<PointsPerSecondText>>,
) {
    let Ok(mut text) = q_text.single_mut() else {
        return;
    };

    text.0 = format!("Points/s: {:>6}", core.pps)
}

pub struct UiStatsPlugin;

impl Plugin for UiStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_stats)
            .add_systems(
                Update,
                (update_points_text, update_points_per_second_text)
                    .run_if(resource_exists::<ProgressionCore>),
            );
    }
}
