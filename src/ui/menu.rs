use bevy::{color::palettes::css::RED, prelude::*, text::FontSmoothing};

use crate::{player::GamingInput, GameAssets, GameState};

const DEFAULT_FONT_SIZE: f32 = 25.0;

// if sounds, than some sliders?
// restart
// export/import save files?

#[derive(Resource, Default)]
struct MenuNavigator {
    highlighted_item: Option<Entity>,
}
#[derive(Component)]
struct MenuScreen;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MenuAction {
    Continue,
}
#[derive(Component)]
struct MenuData {
    action: MenuAction,
}

#[derive(Event)]
pub struct MenuActionEvent {
    pub action: MenuAction,
}

impl MenuAction {
    fn string(self) -> String {
        let s = match self {
            Self::Continue => "Continue",
        };

        s.to_string()
    }
}

fn spawn_button(
    commands: &mut Commands,
    font: Handle<Font>,
    font_size: f32,
    action: MenuAction,
) -> Entity {
    commands
        .spawn((
            Button,
            MenuData { action },
            Text(action.string()),
            TextFont {
                font,
                font_size,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id()
}

fn spawn_background(commands: &mut Commands) -> Entity {
    let root = commands
        .spawn((
            ZIndex(100),
            Node {
                height: Val::Percent(100.0),
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    commands.spawn((
        ChildOf(root),
        ImageNode {
            image: Handle::<Image>::default(),
            color: Color::BLACK.with_alpha(0.65),
            ..default()
        },
        Node {
            height: Val::Percent(100.0),
            width: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
    ));

    commands.spawn((
        ChildOf(root),
        ImageNode {
            image: Handle::<Image>::default(),
            color: Color::BLACK,
            ..default()
        },
        Node {
            height: Val::Percent(70.0),
            width: Val::Percent(30.0),
            position_type: PositionType::Absolute,
            ..default()
        },
    ));

    root
}

fn spawn_buttons(commands: &mut Commands, font: Handle<Font>) -> Entity {
    let back_button = spawn_button(
        commands,
        font.clone(),
        DEFAULT_FONT_SIZE,
        MenuAction::Continue,
    );

    let vertical_buttons = [back_button];

    let buttons = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Vh(0.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            ..default()
        },))
        .add_children(&vertical_buttons)
        .id();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            ZIndex(101),
        ))
        .add_children(&[buttons])
        .id()
}

fn spawn_menu(mut commands: Commands, assets: Res<GameAssets>) {
    let background = spawn_background(&mut commands);
    let button_container = spawn_buttons(&mut commands, assets.pixel_font.clone());

    commands
        .spawn((
            MenuScreen,
            ZIndex(500),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .add_children(&[background, button_container]);
}

fn despawn_menu(mut commands: Commands, q_menu: Query<Entity, With<MenuScreen>>) {
    for entity in &q_menu {
        commands.entity(entity).despawn();
    }
}

fn handle_mouse_movement(
    mut navigator: ResMut<MenuNavigator>,
    q_items: Query<(Entity, &Interaction, &MenuData), With<Button>>,
) {
    navigator.highlighted_item = None;
    for (entity, interaction, _) in &q_items {
        match interaction {
            Interaction::Pressed | Interaction::Hovered => {
                navigator.highlighted_item = Some(entity)
            }
            Interaction::None => {}
        }
    }
}

fn trigger_menu_action(
    navigator: Res<MenuNavigator>,
    gaming_input: Res<GamingInput>,
    q_menu_datas: Query<&MenuData>,
    mut ev_menu_action: EventWriter<MenuActionEvent>,
) {
    let Some(highlighted_item) = navigator.highlighted_item else {
        return;
    };
    if !gaming_input.confirm {
        return;
    }

    let Ok(menu_data) = q_menu_datas.get(highlighted_item) else {
        error!("there is no matching MenuData from Entity, must never happen!");
        return;
    };

    ev_menu_action.write(MenuActionEvent {
        action: menu_data.action,
    });
}

fn reset_all_highlights(mut q_text_colors: Query<&mut TextColor, With<MenuData>>) {
    for mut color in &mut q_text_colors {
        color.0 = Color::WHITE;
    }
}

fn highlight_item(
    navigator: Res<MenuNavigator>,
    mut q_text_colors: Query<&mut TextColor, With<MenuData>>,
) {
    let Some(highlighted_button) = navigator.highlighted_item else {
        return;
    };

    // Highlight color
    if let Ok(mut color) = q_text_colors.get_mut(highlighted_button) {
        color.0 = RED.into();
    }
}

pub struct UiMenuPlugin;

impl Plugin for UiMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MenuActionEvent>()
            .init_resource::<MenuNavigator>()
            .add_systems(OnEnter(GameState::Menu), spawn_menu)
            .add_systems(OnExit(GameState::Menu), despawn_menu)
            .add_systems(
                PostUpdate,
                (
                    reset_all_highlights,
                    handle_mouse_movement,
                    trigger_menu_action,
                    highlight_item,
                )
                    .chain(),
            );
    }
}
