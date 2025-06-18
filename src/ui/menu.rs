use bevy::{
    color::palettes::css::{GRAY, RED},
    prelude::*,
    text::FontSmoothing,
};

use crate::{player::GamingInput, GameAssets, GameState};

const DEFAULT_FONT_SIZE: f32 = 25.0;
const RESET_UNLOCK_TIME: f32 = 3.0;

// if sounds, than some sliders?
// restart
// export/import save files?

#[derive(Resource, Default)]
struct MenuNavigator {
    highlighted_item: Option<Entity>,
}
#[derive(Component)]
struct MenuScreen;
#[derive(Component)]
struct NonInteractable;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MenuAction {
    Continue,
    ResetPopUp,
    Reset,
    CancelReset,
    UnlockReset,
}
#[derive(Component)]
struct MenuData {
    action: MenuAction,
}

#[derive(Component)]
struct ResetPopUp;
#[derive(Component)]
struct ResetPopUpTextTimer {
    timer: Timer,
}
#[derive(Component)]
struct ResetPopUpButton;

#[derive(Event)]
pub struct MenuActionEvent {
    pub action: MenuAction,
}

impl MenuAction {
    fn string(self) -> String {
        let s = match self {
            Self::Continue => "Continue",
            Self::ResetPopUp => "Reset",
            Self::Reset => "Reset",
            Self::CancelReset => "Cancel",
            Self::UnlockReset => "SHOULD NEVER SEE THIS",
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
    let continue_button = spawn_button(
        commands,
        font.clone(),
        DEFAULT_FONT_SIZE,
        MenuAction::Continue,
    );
    let reset_button = spawn_button(
        commands,
        font.clone(),
        DEFAULT_FONT_SIZE,
        MenuAction::ResetPopUp,
    );

    let vertical_buttons = [continue_button, reset_button];

    let buttons = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
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
    q_items: Query<(Entity, &Interaction, &MenuData), (With<Button>, Without<NonInteractable>)>,
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

fn gray_out_items(
    mut q_text_colors: Query<&mut TextColor, (With<MenuData>, With<NonInteractable>)>,
) {
    for mut color in &mut q_text_colors {
        color.0 = GRAY.into();
    }
}

fn spawn_reset_pop_up(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    fn text_font(assets: &GameAssets) -> TextFont {
        TextFont {
            font: assets.pixel_font.clone(),
            font_size: DEFAULT_FONT_SIZE,
            font_smoothing: FontSmoothing::None,
            ..default()
        }
    }

    if !ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::ResetPopUp)
    {
        return;
    }

    let root = commands
        .spawn((
            ResetPopUp,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            ZIndex(1100),
        ))
        .id();

    let width = Val::Percent(70.0);
    let height = Val::Percent(60.0);

    commands.spawn((
        ChildOf(root),
        ZIndex(-1),
        ImageNode {
            image: Handle::<Image>::default(),
            color: Color::BLACK,
            ..default()
        },
        Node {
            width,
            height,
            position_type: PositionType::Absolute,
            ..default()
        },
    ));

    let vertical_flexbox = commands
        .spawn((
            ChildOf(root),
            Node {
                width,
                height,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(80.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    let text_container = commands
        .spawn((
            ChildOf(vertical_flexbox),
            Node {
                width: Val::Percent(80.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .id();

    commands.spawn((
        ChildOf(text_container),
        Text::new("Are you sure?"),
        text_font(&assets),
    ));
    commands.spawn((
        ChildOf(text_container),
        Text::new("This will reset all of your progress."),
        text_font(&assets),
    ));
    commands.spawn((
        ChildOf(text_container),
        ResetPopUpTextTimer {
            timer: Timer::from_seconds(RESET_UNLOCK_TIME, TimerMode::Once),
        },
        Text::new("69.."),
        text_font(&assets),
    ));

    let buttons = commands
        .spawn((
            ChildOf(vertical_flexbox),
            Node {
                width: Val::Percent(80.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .id();

    let reset_button = spawn_button(
        &mut commands,
        assets.pixel_font.clone(),
        DEFAULT_FONT_SIZE,
        MenuAction::Reset,
    );
    commands
        .entity(reset_button)
        .insert(ChildOf(buttons))
        .insert(NonInteractable)
        .insert(ResetPopUpButton);

    let cancel_reset_button = spawn_button(
        &mut commands,
        assets.pixel_font.clone(),
        DEFAULT_FONT_SIZE,
        MenuAction::CancelReset,
    );
    commands
        .entity(cancel_reset_button)
        .insert(ChildOf(buttons));
}

fn close_menu_and_trigger_continue_action(
    gaming_input: Res<GamingInput>,
    q_reset_pop_up: Query<&ResetPopUp>,
    mut ev_menu_action: EventWriter<MenuActionEvent>,
) {
    if !gaming_input.menu {
        return;
    }

    // The Reset Pop Up is open, that means we don't want to close the entire menu, just the pop
    // up. I decided to NOT use state machine (which would be much better for clean code) because
    // it would be overkill, I don't want to have any big state, so this should work for the entire
    // game.
    if !q_reset_pop_up.is_empty() {
        return;
    }

    ev_menu_action.write(MenuActionEvent {
        action: MenuAction::Continue,
    });
}

fn despawn_reset_pop_up(
    mut commands: Commands,
    gaming_input: Res<GamingInput>,
    q_reset_pop_up: Query<Entity, With<ResetPopUp>>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    let event = ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::CancelReset);
    let input = gaming_input.menu || gaming_input.cancel;

    let Ok(entity) = q_reset_pop_up.single() else {
        return;
    };

    if input || event {
        commands.entity(entity).despawn();
    }
}

fn update_reset_pop_up_timer_text(
    time: Res<Time>,
    mut q_pop_up_text: Query<(&mut Text, &mut ResetPopUpTextTimer)>,
    mut ev_menu_action: EventWriter<MenuActionEvent>,
) {
    let Ok((mut text, mut pop_up)) = q_pop_up_text.single_mut() else {
        return;
    };

    pop_up.timer.tick(time.delta());

    let remaining_time = pop_up.timer.duration().as_secs_f32() - pop_up.timer.elapsed_secs();
    text.0 = format!("Wait {:.1}", remaining_time);

    if pop_up.timer.just_finished() {
        text.0 = "I warned you...".to_string();
        ev_menu_action.write(MenuActionEvent {
            action: MenuAction::UnlockReset,
        });
    }
}

fn remove_reset_pop_up_timer_component(
    mut commands: Commands,
    q_text: Query<Entity, With<ResetPopUpTextTimer>>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    let Ok(entity) = q_text.single() else {
        return;
    };

    if ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::UnlockReset)
    {
        commands.entity(entity).remove::<ResetPopUpTextTimer>();
    }
}

fn remove_non_interactable_from_reset_button(
    mut commands: Commands,
    q_reset_button: Query<Entity, (With<NonInteractable>, With<ResetPopUpButton>)>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    let Ok(entity) = q_reset_button.single() else {
        return;
    };

    if ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::UnlockReset)
    {
        commands.entity(entity).remove::<NonInteractable>();
    }
}

fn set_buttons_inactive_on_reset_pop_up(
    mut commands: Commands,
    q_buttons: Query<(Entity, &MenuData)>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    if ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::ResetPopUp)
    {
        for (entity, menu_data) in &q_buttons {
            match menu_data.action {
                MenuAction::Reset | MenuAction::CancelReset => continue,
                _ => {}
            }

            commands.entity(entity).insert(NonInteractable);
        }
    }
}

fn set_buttons_active_on_reset_cancel(
    mut commands: Commands,
    q_buttons: Query<(Entity, &MenuData)>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    if ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::CancelReset)
    {
        for (entity, menu_data) in &q_buttons {
            match menu_data.action {
                MenuAction::Reset | MenuAction::CancelReset => continue,
                _ => {}
            }
            commands.entity(entity).remove::<NonInteractable>();
        }
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
                    gray_out_items,
                    spawn_reset_pop_up,
                    close_menu_and_trigger_continue_action.run_if(in_state(GameState::Menu)),
                    despawn_reset_pop_up,
                    update_reset_pop_up_timer_text,
                    remove_reset_pop_up_timer_component,
                    remove_non_interactable_from_reset_button,
                    set_buttons_active_on_reset_cancel,
                    set_buttons_inactive_on_reset_pop_up,
                )
                    .chain()
                    .run_if(resource_exists::<GameAssets>),
            );
    }
}
