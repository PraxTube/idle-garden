use bevy::{
    color::palettes::css::{GRAY, RED},
    prelude::*,
    text::FontSmoothing,
};

use crate::{player::GamingInput, world::ProgressionCore, GameAssets, GameState};

use super::Consent;

const DEFAULT_FONT_SIZE: f32 = 25.0;
const RESET_UNLOCK_TIME: f32 = 3.0;
const DISCORD_LINK: &str = "https://discord.gg/2h7dncQNTr";

#[derive(Resource, Default)]
struct MenuNavigator {
    highlighted_item: Option<Entity>,
}
#[derive(Component)]
struct MenuScreen;
#[derive(Component)]
struct NonInteractable;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MenuAction {
    Continue,
    SendDataYes,
    SendDataNo,
    MusicOn,
    MusicOff,
    SoundOn,
    SoundOff,
    ResetPopUp,
    Reset,
    CancelReset,
    UnlockReset,
    Discord,
}
#[derive(Component, Debug)]
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

#[derive(Component)]
struct DiscordButton {
    timer: Timer,
    delay: Timer,
}

impl MenuAction {
    fn string(self) -> String {
        let s = match self {
            Self::Continue => "Continue",
            Self::SendDataYes => "Send Data=Y",
            Self::SendDataNo => "Send Data=N",
            Self::MusicOn => "Music=On",
            Self::MusicOff => "Music=Off",
            Self::SoundOn => "Sound=On",
            Self::SoundOff => "Sound=Off",
            Self::ResetPopUp => "Reset",
            Self::Reset => "Reset",
            Self::CancelReset => "Cancel",
            Self::UnlockReset => "SHOULD NEVER SEE THIS",
            Self::Discord => "SHOULD NEVER SEE THIS",
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

fn spawn_discord_button(commands: &mut Commands, assets: &Res<GameAssets>) -> Entity {
    commands
        .spawn((
            MenuData {
                action: MenuAction::Discord,
            },
            DiscordButton {
                timer: Timer::from_seconds(0.5, TimerMode::Once),
                delay: Timer::from_seconds(0.5, TimerMode::Once),
            },
            Button,
            ZIndex(102),
            Node {
                width: Val::Px(100.0),
                height: Val::Px(100.0),
                bottom: Val::Px(20.0),
                right: Val::Px(20.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            ImageNode {
                image: assets.discord_button.clone(),
                color: Color::WHITE.with_alpha(0.0),
                ..default()
            },
        ))
        .id()
}

fn spawn_background(commands: &mut Commands, assets: &GameAssets) -> Entity {
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
        ZIndex(-1),
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
            image: assets.menu_background.clone(),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
    ));

    root
}

fn spawn_buttons(
    commands: &mut Commands,
    font: Handle<Font>,
    consent: bool,
    core: &ProgressionCore,
) -> Entity {
    let continue_button = spawn_button(
        commands,
        font.clone(),
        DEFAULT_FONT_SIZE,
        MenuAction::Continue,
    );
    let music_button = spawn_button(
        commands,
        font.clone(),
        DEFAULT_FONT_SIZE,
        if core.music {
            MenuAction::MusicOn
        } else {
            MenuAction::MusicOff
        },
    );
    let sound_button = spawn_button(
        commands,
        font.clone(),
        DEFAULT_FONT_SIZE,
        if core.sound {
            MenuAction::SoundOn
        } else {
            MenuAction::SoundOff
        },
    );
    let send_data = spawn_button(
        commands,
        font.clone(),
        DEFAULT_FONT_SIZE,
        if consent {
            MenuAction::SendDataYes
        } else {
            MenuAction::SendDataNo
        },
    );
    let reset_button = spawn_button(
        commands,
        font.clone(),
        DEFAULT_FONT_SIZE,
        MenuAction::ResetPopUp,
    );

    let vertical_buttons = [
        continue_button,
        music_button,
        sound_button,
        send_data,
        reset_button,
    ];

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

fn spawn_menu(
    mut commands: Commands,
    assets: Res<GameAssets>,
    core: Res<ProgressionCore>,
    consent: Res<Consent>,
) {
    let background = spawn_background(&mut commands, &assets);
    let button_container =
        spawn_buttons(&mut commands, assets.pixel_font.clone(), consent.0, &core);
    let discord_button = spawn_discord_button(&mut commands, &assets);

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
        .add_children(&[background, button_container, discord_button]);
}

fn despawn_menu(
    mut commands: Commands,
    q_menus: Query<Entity, Or<(With<MenuScreen>, With<ResetPopUp>)>>,
) {
    for entity in &q_menus {
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

fn reset_all_highlights(
    mut q_text_colors: Query<&mut TextColor, With<MenuData>>,
    mut q_images: Query<&mut ImageNode, With<MenuData>>,
) {
    for mut color in &mut q_text_colors {
        color.0 = Color::WHITE;
    }
    for mut image in &mut q_images {
        image.color = Color::WHITE;
    }
}

fn highlight_item(
    navigator: Res<MenuNavigator>,
    mut q_text_colors: Query<&mut TextColor, With<MenuData>>,
    mut q_images: Query<&mut ImageNode, With<MenuData>>,
) {
    let Some(highlighted_button) = navigator.highlighted_item else {
        return;
    };

    if let Ok(mut color) = q_text_colors.get_mut(highlighted_button) {
        color.0 = RED.into();
    }
    if let Ok(mut image) = q_images.get_mut(highlighted_button) {
        image.color = RED.into();
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

    let width = Val::Percent(65.0);
    let height = Val::Percent(60.0);

    commands.spawn((
        ChildOf(root),
        ZIndex(-1),
        ImageNode {
            image: assets.reset_pop_up_background.clone(),
            ..default()
        },
        Node {
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
                width: Val::Percent(90.0),
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

fn trigger_close_reset_pop_up_event(
    gaming_input: Res<GamingInput>,
    mut ev_menu_action: EventWriter<MenuActionEvent>,
) {
    let input = gaming_input.menu || gaming_input.cancel;

    if input {
        ev_menu_action.write(MenuActionEvent {
            action: MenuAction::CancelReset,
        });
    }
}

fn despawn_reset_pop_up(
    mut commands: Commands,
    q_reset_pop_up: Query<Entity, With<ResetPopUp>>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    if !ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::CancelReset)
    {
        return;
    }

    for entity in &q_reset_pop_up {
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

fn set_buttons_active_on_cancel_reset(
    mut commands: Commands,
    q_buttons: Query<(Entity, &MenuData)>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    if !ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::CancelReset)
    {
        return;
    }

    for (entity, menu_data) in &q_buttons {
        match menu_data.action {
            MenuAction::Reset | MenuAction::CancelReset => continue,
            _ => {}
        }
        commands.entity(entity).remove::<NonInteractable>();
    }
}

fn toggle_send_data_button(
    mut q_texts: Query<(&mut Text, &mut MenuData)>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    let mut toggle_on = false;
    let mut toggle_off = false;

    for ev in ev_menu_action.read() {
        if ev.action == MenuAction::SendDataYes {
            toggle_on = true;
        }
        if ev.action == MenuAction::SendDataNo {
            toggle_off = true;
        }
    }

    debug_assert!(!(toggle_on && toggle_off));

    if toggle_off {
        for (mut text, mut data) in &mut q_texts {
            if data.action == MenuAction::SendDataNo {
                data.action = MenuAction::SendDataYes;
                text.0 = MenuAction::SendDataYes.string();
            }
        }
    } else if toggle_on {
        for (mut text, mut data) in &mut q_texts {
            if data.action == MenuAction::SendDataYes {
                data.action = MenuAction::SendDataNo;
                text.0 = MenuAction::SendDataNo.string();
            }
        }
    }
}

fn toggle_music(
    mut q_texts: Query<(&mut Text, &mut MenuData)>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    let mut toggle_on = false;
    let mut toggle_off = false;

    for ev in ev_menu_action.read() {
        if ev.action == MenuAction::MusicOn {
            toggle_on = true;
        }
        if ev.action == MenuAction::MusicOff {
            toggle_off = true;
        }
    }

    debug_assert!(!(toggle_on && toggle_off));

    if toggle_off {
        for (mut text, mut data) in &mut q_texts {
            if data.action == MenuAction::MusicOff {
                data.action = MenuAction::MusicOn;
                text.0 = MenuAction::MusicOn.string();
            }
        }
    } else if toggle_on {
        for (mut text, mut data) in &mut q_texts {
            if data.action == MenuAction::MusicOn {
                data.action = MenuAction::MusicOff;
                text.0 = MenuAction::MusicOff.string();
            }
        }
    }
}

fn toggle_sound(
    mut q_texts: Query<(&mut Text, &mut MenuData)>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    let mut toggle_on = false;
    let mut toggle_off = false;

    for ev in ev_menu_action.read() {
        if ev.action == MenuAction::SoundOn {
            toggle_on = true;
        }
        if ev.action == MenuAction::SoundOff {
            toggle_off = true;
        }
    }

    debug_assert!(!(toggle_on && toggle_off));

    if toggle_off {
        for (mut text, mut data) in &mut q_texts {
            if data.action == MenuAction::SoundOff {
                data.action = MenuAction::SoundOn;
                text.0 = MenuAction::SoundOn.string();
            }
        }
    } else if toggle_on {
        for (mut text, mut data) in &mut q_texts {
            if data.action == MenuAction::SoundOn {
                data.action = MenuAction::SoundOff;
                text.0 = MenuAction::SoundOff.string();
            }
        }
    }
}

fn animate_discord_button(
    time: Res<Time>,
    mut q_discord_button: Query<(&mut ImageNode, &mut DiscordButton)>,
) {
    let Ok((mut image, mut discord_button)) = q_discord_button.single_mut() else {
        return;
    };

    if !discord_button.delay.finished() {
        image.color.set_alpha(0.0);
        discord_button.delay.tick(time.delta());
        return;
    }

    discord_button.timer.tick(time.delta());

    if discord_button.timer.just_finished() {
        image.color.set_alpha(1.0);
        return;
    }

    if discord_button.timer.finished() {
        return;
    }

    let alpha = discord_button.timer.elapsed_secs() / discord_button.timer.duration().as_secs_f32();
    image.color.set_alpha(alpha);
}

fn open_discord_link(mut ev_menu_action: EventReader<MenuActionEvent>) {
    if !ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::Discord)
    {
        return;
    }

    let err_str = "failed to open discord link in default browser";

    #[cfg(not(target_arch = "wasm32"))]
    if open::that(DISCORD_LINK).is_err() {
        error!(err_str);
    }

    #[cfg(target_arch = "wasm32")]
    if let Some(win) = web_sys::window() {
        if win
            .open_with_url_and_target(DISCORD_LINK, "_blank")
            .is_err()
        {
            error!(err_str);
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
                    trigger_close_reset_pop_up_event,
                    despawn_reset_pop_up,
                    toggle_send_data_button,
                    toggle_music,
                    toggle_sound,
                    update_reset_pop_up_timer_text,
                    remove_reset_pop_up_timer_component,
                    remove_non_interactable_from_reset_button,
                    animate_discord_button,
                    open_discord_link,
                    set_buttons_active_on_cancel_reset,
                    set_buttons_inactive_on_reset_pop_up,
                )
                    .chain()
                    .run_if(resource_exists::<GameAssets>),
            );
    }
}
