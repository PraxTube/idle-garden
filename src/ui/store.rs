use bevy::{
    color::palettes::css::{DARK_GRAY, RED},
    prelude::*,
    text::FontSmoothing,
    ui::RelativeCursorPosition,
};

use crate::{
    player::GamingInput,
    world::{utils::format_money_string, Flora, MapData, ProgressionCore},
    GameAssets, GameState, DEFAULT_WINDOW_WIDTH,
};

use super::outline::TextOutline;

const STORE_ROOT_PADDING_VERTICAL: f32 = 40.0;
const HORIZONTAL_ITEM_PADDING: f32 = 50.0;
const NUMBER_OF_ITEMS_ON_PAGE: usize = 8;

#[derive(Component)]
struct StoreRoot;
#[derive(Component)]
struct StoreItemContainer;
#[derive(Component)]
struct StoreItem {
    index: usize,
}
#[derive(Component)]
struct ItemIcon;
#[derive(Component)]
struct ItemCountText;
#[derive(Component)]
struct ItemCostText;
#[derive(Component)]
struct ItemUnaffordableOverlay;

#[derive(Resource, Default)]
struct StorePageItems {
    items: [Flora; NUMBER_OF_ITEMS_ON_PAGE],
    is_affordable: [bool; NUMBER_OF_ITEMS_ON_PAGE],
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
struct Navigator {
    highlighted_item: Option<Entity>,
}

fn spawn_store_item(
    commands: &mut Commands,
    assets: &GameAssets,
    items_container: Entity,
    item: StoreItem,
) {
    let item_root = commands
        .spawn((
            ChildOf(items_container),
            Button,
            item,
            Node {
                height: Val::Percent(50.0),
                aspect_ratio: Some(1.0),
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode {
                image: assets.store_item_background.clone(),
                ..default()
            },
        ))
        .id();

    commands.spawn((
        ChildOf(item_root),
        ItemIcon,
        Node {
            height: Val::Percent(65.0),
            aspect_ratio: Some(1.0),
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
        ItemCountText,
        Node {
            bottom: Val::Px(0.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        TextOutline::new(
            "x69".to_string(),
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
        ZIndex(3),
    ));

    commands.spawn((
        ItemCostText,
        ChildOf(item_root),
        Node {
            top: Val::Px(-50.0),
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
                font_size: 25.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            true,
        ),
        ZIndex(3),
    ));

    commands.spawn((
        ChildOf(item_root),
        ItemUnaffordableOverlay,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_self: AlignSelf::Center,
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

fn spawn_store(mut commands: Commands, assets: Res<GameAssets>, images: Res<Assets<Image>>) {
    let store_bar_image = images
        .get(&assets.store_bar)
        .expect("failed to get store bar image");
    let store_bar_image_size = store_bar_image.size();

    let width = store_bar_image_size.x as f32;
    let height = store_bar_image_size.y as f32;
    let horizontal_padding = (DEFAULT_WINDOW_WIDTH - width) * 0.5;
    debug_assert!(horizontal_padding > 0.0);

    let root = commands
        .spawn((
            StoreRoot,
            RelativeCursorPosition::default(),
            ImageNode {
                image: assets.store_bar.clone(),
                ..default()
            },
            Node {
                left: Val::Px(horizontal_padding),
                right: Val::Px(horizontal_padding),
                bottom: Val::Px(STORE_ROOT_PADDING_VERTICAL),
                width: Val::Px(width),
                height: Val::Px(height),
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

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
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    for index in 0..NUMBER_OF_ITEMS_ON_PAGE {
        spawn_store_item(&mut commands, &assets, items_container, StoreItem { index });
    }
}

fn update_item_affordability(
    core: Res<ProgressionCore>,
    map_data: Res<MapData>,
    mut store_page: ResMut<StorePageItems>,
) {
    for i in 0..NUMBER_OF_ITEMS_ON_PAGE {
        store_page.is_affordable[i] = core.is_affordable(&map_data, &store_page.items[i]);
    }
}

fn reset_all_highlights(mut q_items: Query<&mut ImageNode, With<StoreItem>>) {
    for mut image in &mut q_items {
        image.color = Color::WHITE;
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

fn highlight_item(q_navigator: Query<&Navigator>, mut q_items: Query<&mut ImageNode>) {
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

fn toggle_item_unaffordable_overlay(
    store_page: Res<StorePageItems>,
    q_items: Query<&StoreItem>,
    mut q_overlays: Query<(&ChildOf, &mut Visibility), With<ItemUnaffordableOverlay>>,
) {
    for (parent, mut visibility) in &mut q_overlays {
        let Ok(item) = q_items.get(parent.0) else {
            continue;
        };

        *visibility = if store_page.is_affordable[item.index] {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

fn hide_item_cost_texts(mut q_cost_texts: Query<&mut Visibility, With<ItemCostText>>) {
    for mut visibility in &mut q_cost_texts {
        *visibility = Visibility::Hidden;
    }
}

fn update_item_cost_text(
    core: Res<ProgressionCore>,
    map_data: Res<MapData>,
    store_page: Res<StorePageItems>,
    q_navigator: Query<&Navigator>,
    q_items: Query<&StoreItem>,
    mut q_cost_texts: Query<(&ChildOf, &mut Visibility, &mut TextOutline), With<ItemCostText>>,
) {
    let Ok(navigator) = q_navigator.single() else {
        return;
    };

    let Some(highlighted_item) = navigator.highlighted_item else {
        return;
    };

    for (parent, mut visibility, mut outline) in &mut q_cost_texts {
        if parent.parent() != highlighted_item {
            continue;
        }

        let Ok(store_item) = q_items.get(highlighted_item) else {
            continue;
        };

        let color = if store_page.is_affordable[store_item.index] {
            Color::WHITE
        } else {
            RED.into()
        };

        let item = store_page.items[store_item.index];
        let cost = map_data
            .flora_data(item.index())
            .cost(core.flora[item.index()].into());

        outline.text = format_money_string(cost.into());
        outline.color = color;

        *visibility = Visibility::Inherited;
    }
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
    mut q_outlines: Query<&mut TextOutline, With<ItemCountText>>,
) {
    for (children, item) in &q_items {
        for child in children {
            let Ok(mut outline) = q_outlines.get_mut(*child) else {
                continue;
            };

            outline.text = format!(
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
    mut q_image_nodes: Query<&mut ImageNode, With<ItemIcon>>,
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
    store_page.items[3] = Flora::SwampTree;
    store_page.items[6] = Flora::Sunflower;
    store_page.items[7] = Flora::Tree;
}

pub struct UiStorePlugin;

impl Plugin for UiStorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ItemPressed>()
            .init_resource::<StorePageItems>()
            .add_systems(OnExit(GameState::AssetLoading), spawn_store)
            .add_systems(
                PreUpdate,
                update_item_affordability
                    .run_if(resource_exists::<ProgressionCore>.and(resource_exists::<MapData>)),
            )
            .add_systems(
                Update,
                (
                    reset_all_highlights,
                    handle_button_interaction.run_if(in_state(GameState::Gaming)),
                    highlight_item,
                    trigger_button_pressed,
                    update_store_item_count_texts.run_if(resource_exists::<ProgressionCore>),
                    toggle_item_unaffordable_overlay,
                    hide_item_cost_texts,
                    update_item_cost_text.run_if(resource_exists::<MapData>),
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
