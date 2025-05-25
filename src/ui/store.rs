use bevy::{
    color::palettes::css::{DARK_GRAY, GRAY},
    prelude::*,
    text::FontSmoothing,
};

use crate::{
    player::GamingInput,
    world::{Flora, ProgressionCore},
    GameAssets, GameState, DEFAULT_WINDOW_WIDTH,
};

const STORE_ROOT_PADDING_VERTICAL: f32 = 40.0;
const STORE_ROOT_PADDING_HORIZONTAL: f32 = 100.0;
const STORE_HEIGHT: f32 = 75.0;
const HORIZONTAL_ITEM_PADDING: f32 = 30.0;
const NUMBER_OF_ITEMS_ON_PAGE: usize = 10;

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
#[derive(Component)]
struct StoreItemIcon;
#[derive(Component)]
struct StoreItemCountText;

#[derive(Resource, Default)]
struct StorePageItems {
    items: [Flora; NUMBER_OF_ITEMS_ON_PAGE],
}

#[derive(Event)]
pub struct ItemPressed {
    pub flora: Flora,
}

impl StorePageItems {
    fn get_by_index(&self, index: usize) -> Flora {
        if index >= NUMBER_OF_ITEMS_ON_PAGE {
            error!("trying to index store page items with index: {}, but only has: {} items total. Should never happen.", index, NUMBER_OF_ITEMS_ON_PAGE);
            return Flora::default();
        }

        self.items[index]
    }
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
                left: Val::Px(STORE_ROOT_PADDING_HORIZONTAL),
                right: Val::Px(STORE_ROOT_PADDING_HORIZONTAL),
                top: Val::Px(STORE_ROOT_PADDING_VERTICAL),
                height: Val::Px(STORE_HEIGHT),
                width: Val::Px(DEFAULT_WINDOW_WIDTH - 2.0 * STORE_ROOT_PADDING_HORIZONTAL),
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
                padding: UiRect {
                    left: Val::Px(HORIZONTAL_ITEM_PADDING),
                    right: Val::Px(HORIZONTAL_ITEM_PADDING),
                    ..default()
                },
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Auto,
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    for index in 0..NUMBER_OF_ITEMS_ON_PAGE {
        let item_root = commands
            .spawn((
                ChildOf(items_container),
                Button,
                StoreItem { index },
                Node {
                    height: Val::Percent(70.0),
                    aspect_ratio: Some(1.0),
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ImageNode {
                    color: GRAY.into(),
                    image: Handle::<Image>::default(),
                    ..default()
                },
            ))
            .id();

        commands.spawn((
            ChildOf(item_root),
            StoreItemIcon,
            Node {
                width: Val::Percent(80.0),
                height: Val::Percent(80.0),
                align_self: AlignSelf::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            ImageNode {
                image: Flora::default().icon(&assets),
                ..default()
            },
            ZIndex(1),
        ));

        commands.spawn((
            ChildOf(item_root),
            StoreItemCountText,
            Text("x300".to_string()),
            Node {
                bottom: Val::Px(-20.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            TextFont {
                font: assets.pixel_font.clone(),
                font_size: 20.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            TextColor(Color::BLACK),
            ZIndex(2),
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
    store_page: Res<StorePageItems>,
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

    let flora = store_page.get_by_index(item.index);

    ev_item_pressed.write(ItemPressed { flora });
}

fn update_store_item_count_texts(
    core: Res<ProgressionCore>,
    store_page: Res<StorePageItems>,
    q_items: Query<(&Children, &StoreItem)>,
    mut q_texts: Query<&mut Text, With<StoreItemCountText>>,
) {
    for (children, item) in &q_items {
        for child in children {
            let Ok(mut text) = q_texts.get_mut(*child) else {
                continue;
            };

            text.0 = format!(
                "x{}",
                core.flora[store_page.get_by_index(item.index).index()]
            );
        }
    }
}

fn update_store_item_icons(
    assets: Res<GameAssets>,
    store_page: Res<StorePageItems>,
    q_items: Query<(&Children, &StoreItem)>,
    mut q_image_nodes: Query<&mut ImageNode, With<StoreItemIcon>>,
) {
    for (children, item) in &q_items {
        for child in children {
            let Ok(mut image_node) = q_image_nodes.get_mut(*child) else {
                continue;
            };

            image_node.image = store_page.get_by_index(item.index).icon(&assets);
        }
    }
}

fn update_store_page_items(mut store_page: ResMut<StorePageItems>) {
    store_page.items[0] = Flora::Potatoe;
    store_page.items[1] = Flora::Raddish;
    store_page.items[2] = Flora::Carrot;
    store_page.items[6] = Flora::Sunflower;
}

pub struct UiStorePlugin;

impl Plugin for UiStorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ItemPressed>()
            .init_resource::<StorePageItems>()
            .add_systems(OnExit(GameState::AssetLoading), spawn_store)
            .add_systems(
                Update,
                (
                    reset_all_highlights,
                    handle_button_interaction,
                    highlight_item,
                    trigger_button_pressed,
                    update_store_item_count_texts.run_if(resource_exists::<ProgressionCore>),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                update_store_item_icons
                    .run_if(resource_changed::<StorePageItems>.and(resource_exists::<GameAssets>)),
            )
            .add_systems(OnExit(GameState::AssetLoading), update_store_page_items);
    }
}
