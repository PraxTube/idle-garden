use bevy::{
    color::palettes::css::{DARK_GRAY, GRAY},
    prelude::*,
};

use crate::{player::GamingInput, world::Flora, GameAssets, GameState, DEFAULT_WINDOW_WIDTH};

const STORE_ROOT_PADDING: f32 = 40.0;

#[derive(Component)]
struct StoreRoot;
#[derive(Component)]
struct StoreBackground;
#[derive(Component)]
struct StoreItemContainer;
#[derive(Component)]
struct StoreItem {
    index: usize,
}

#[derive(Event)]
pub struct ItemPressed {
    pub flora: Flora,
}

#[derive(Component, Default)]
pub struct Navigator {
    /// The currently highlighted item (ready to be triggered).
    /// If you don't want to start with any item selected from the start, leave this empty.
    pub highlighted_item: Option<Entity>,
}

fn spawn_store(mut commands: Commands, assets: Res<GameAssets>) {
    let root = commands
        .spawn((
            StoreRoot,
            Node {
                left: Val::Px(STORE_ROOT_PADDING),
                right: Val::Px(STORE_ROOT_PADDING),
                top: Val::Px(STORE_ROOT_PADDING),
                height: Val::Px(125.0),
                width: Val::Px(DEFAULT_WINDOW_WIDTH - 2.0 * STORE_ROOT_PADDING),
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    commands.spawn((
        ChildOf(root),
        StoreBackground,
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

    let items_container = commands
        .spawn((
            ChildOf(root),
            StoreItemContainer,
            Navigator::default(),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Auto,
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    for index in 0..8 {
        commands.spawn((
            ChildOf(items_container),
            Button,
            StoreItem { index },
            ImageNode {
                color: GRAY.into(),
                image: Handle::<Image>::default(),
                ..default()
            },
        ));
    }
}

fn reset_all_highlights(mut q_items: Query<&mut ImageNode, With<StoreItem>>) {
    for mut image in &mut q_items {
        image.color = GRAY.into();
    }
}

fn handle_button_interaction(
    mut q_navigator: Query<&mut Navigator>,
    q_items: Query<(Entity, &Interaction), (With<Button>, With<StoreItem>)>,
) {
    let Ok(mut navigator) = q_navigator.single_mut() else {
        return;
    };

    navigator.highlighted_item = None;

    for (entity, interaction) in &q_items {
        match interaction {
            Interaction::Pressed | Interaction::Hovered => {
                navigator.highlighted_item = Some(entity)
            }
            Interaction::None => {}
        }
    }
}

fn highlight_item(
    q_navigator: Query<&Navigator>,
    mut q_items: Query<&mut ImageNode, With<StoreItem>>,
) {
    let Ok(navigator) = q_navigator.single() else {
        return;
    };
    let Some(highlighted_item) = navigator.highlighted_item else {
        return;
    };

    let Ok(mut image) = q_items.get_mut(highlighted_item) else {
        error!("highlighted item doesn't match with any of the item query, should never happen!");
        return;
    };

    image.color = DARK_GRAY.into();
}

fn trigger_button_pressed(
    gaming_input: Res<GamingInput>,
    q_navigator: Query<&Navigator>,
    q_items: Query<&StoreItem>,
    mut ev_item_pressed: EventWriter<ItemPressed>,
) {
    if !gaming_input.confirm {
        return;
    }

    let Ok(navigator) = q_navigator.single() else {
        return;
    };

    let Some(highlighted_item) = navigator.highlighted_item else {
        return;
    };

    let Ok(item) = q_items.get(highlighted_item) else {
        error!(
            "highlighted item in navigator doesn't match with any item query, should never happen!"
        );
        return;
    };

    let vegetation = match item.index {
        0 => Flora::Potatoe,
        1 => Flora::Tree,
        _ => Flora::Weed,
    };

    ev_item_pressed.write(ItemPressed { flora: vegetation });
}

pub struct UiStorePlugin;

impl Plugin for UiStorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ItemPressed>()
            .add_systems(OnExit(GameState::AssetLoading), spawn_store)
            .add_systems(
                Update,
                (
                    reset_all_highlights,
                    handle_button_interaction,
                    highlight_item,
                    trigger_button_pressed,
                )
                    .chain(),
            );
    }
}
