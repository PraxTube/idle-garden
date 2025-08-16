use bevy::{
    color::palettes::css::{DARK_GRAY, RED},
    prelude::*,
    text::FontSmoothing,
};

use crate::{
    player::GamingInput,
    world::{utils::format_money_string, ProgressionCore, POINTS_CAP_COST_INCREASE_PER_SILO},
    GameAssets, GameState,
};

use super::outline::TextOutline;

#[derive(Component)]
struct StatsRoot;
#[derive(Component)]
struct PointsText;
#[derive(Component, Default)]
struct CapIncrease {
    highlighted: bool,
}
#[derive(Component)]
struct CapIncreaseCostText;
#[derive(Component)]
struct PointsPerSecondText;
#[derive(Component)]
struct UnaffordableOverlay;

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
            position_type: PositionType::Absolute,
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
            position_type: PositionType::Absolute,
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

    let cap_increase_button = commands
        .spawn((
            ChildOf(container),
            CapIncrease::default(),
            Button,
            ImageNode {
                image: assets.store_item_background.clone(),
                ..default()
            },
            Node {
                left: Val::Px(0.0),
                top: Val::Px(120.0),
                width: Val::Px(160.0),
                height: Val::Px(80.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .id();

    commands.spawn((
        ChildOf(cap_increase_button),
        Node {
            top: Val::Px(15.0),
            ..default()
        },
        TextOutline::new(
            "+Cap".to_string(),
            1.0,
            Color::WHITE,
            Color::BLACK,
            TextFont {
                font: assets.pixel_font.clone(),
                font_size: 35.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            true,
        ),
    ));

    commands.spawn((
        CapIncreaseCostText,
        ChildOf(cap_increase_button),
        Node {
            bottom: Val::Px(-25.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        Visibility::Hidden,
        TextOutline::new(
            "$431".to_string(),
            1.0,
            Color::WHITE,
            Color::BLACK,
            TextFont {
                font: assets.pixel_font.clone(),
                font_size: 30.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            true,
        ),
        ZIndex(3),
    ));

    commands.spawn((
        UnaffordableOverlay,
        ChildOf(cap_increase_button),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        Visibility::Hidden,
        ImageNode {
            image: assets.store_item_unaffordable_overlay.clone(),
            ..default()
        },
        ZIndex(3),
    ));
}

fn update_points_text(
    core: Res<ProgressionCore>,
    mut q_text: Query<&mut TextOutline, With<PointsText>>,
) {
    let Ok(mut outline) = q_text.single_mut() else {
        return;
    };

    outline.text = format!(
        "{}/{}",
        format_money_string(core.points),
        format_money_string(core.points_cap)
    );
}

fn update_points_per_second_text(
    core: Res<ProgressionCore>,
    mut q_text: Query<&mut TextOutline, With<PointsPerSecondText>>,
) {
    let Ok(mut outline) = q_text.single_mut() else {
        return;
    };

    outline.text = format_money_string(core.pps.into()) + "/s";
}

fn handle_cap_increase_button_interaction(
    q_cap_increase: Single<(&mut CapIncrease, &Interaction), With<Button>>,
) {
    let (mut cap_increase, interaction) = q_cap_increase.into_inner();

    match interaction {
        Interaction::Pressed => cap_increase.highlighted = true,
        Interaction::Hovered => cap_increase.highlighted = true,
        Interaction::None => cap_increase.highlighted = false,
    }
}

fn update_cap_increase_visuals(
    core: Res<ProgressionCore>,
    q_cap_increase: Single<(&mut ImageNode, &CapIncrease)>,
    q_cost_text: Single<(&mut Visibility, &mut TextOutline), With<CapIncreaseCostText>>,
    q_unaffordable_overlay: Single<
        &mut Visibility,
        (With<UnaffordableOverlay>, Without<CapIncreaseCostText>),
    >,
) {
    let (mut image, cap_increase) = q_cap_increase.into_inner();
    let (mut cost_visibility, mut cost_text) = q_cost_text.into_inner();
    let mut unaffordable_overlay_visibility = q_unaffordable_overlay.into_inner();

    (image.color, *cost_visibility) = if cap_increase.highlighted {
        (DARK_GRAY.into(), Visibility::Inherited)
    } else {
        (Color::WHITE, Visibility::Hidden)
    };

    let cost = core.silos * POINTS_CAP_COST_INCREASE_PER_SILO;
    cost_text.text = format_money_string(cost);
    (cost_text.color, *unaffordable_overlay_visibility) = if cost > core.points {
        (RED.into(), Visibility::Inherited)
    } else {
        (Color::WHITE, Visibility::Hidden)
    };
}

fn buy_silo(
    gaming_input: Res<GamingInput>,
    mut core: ResMut<ProgressionCore>,
    q_cap_increase: Single<&CapIncrease>,
) {
    if !gaming_input.confirm {
        return;
    }

    let cap_increase = q_cap_increase.into_inner();

    if !cap_increase.highlighted {
        return;
    }

    let cost = core.silos * POINTS_CAP_COST_INCREASE_PER_SILO;
    if core.points < cost {
        return;
    }

    core.silos += 1;
    core.points -= cost;
}

pub struct UiStatsPlugin;

impl Plugin for UiStatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_stats)
            .add_systems(
                Update,
                (
                    update_points_text,
                    update_points_per_second_text,
                    handle_cap_increase_button_interaction,
                    update_cap_increase_visuals,
                    buy_silo,
                )
                    .chain()
                    .run_if(resource_exists::<ProgressionCore>),
            );
    }
}
