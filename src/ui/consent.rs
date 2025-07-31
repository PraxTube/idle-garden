use bevy::{
    color::palettes::tailwind::{GRAY_500, GRAY_700},
    prelude::*,
    text::FontSmoothing,
};

use crate::{GameAssets, GameState};

const CONSENT_NOTICE: &str = "The game collects user data for the purpose of my Bachelor Thesis.\n\nThe data is used for research only.\n\nRead which data is being tracked on the Itch page. You can disable this in the settings.";

#[derive(Component)]
struct OkayButton;
#[derive(Component)]
struct ConsentRoot;

fn spawn_consent(mut commands: Commands, assets: Res<GameAssets>) {
    let background = commands
        .spawn((
            ConsentRoot,
            GlobalZIndex(100000),
            ImageNode {
                image: Handle::<Image>::default(),
                color: Color::BLACK.with_alpha(0.6),
                ..default()
            },
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    let root = commands
        .spawn((
            ChildOf(background),
            Node {
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                width: Val::Px(0.0),
                height: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    let canvas = commands
        .spawn((
            ChildOf(root),
            ImageNode {
                image: Handle::<Image>::default(),
                color: Color::BLACK,
                ..default()
            },
            Node {
                width: Val::Px(800.0),
                height: Val::Px(450.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    commands.spawn((
        ChildOf(canvas),
        Text::new("Consent Notice"),
        TextFont {
            font: assets.pixel_font.clone(),
            font_size: 30.0,
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        Node {
            width: Val::Percent(85.0),
            top: Val::Px(30.0),
            position_type: PositionType::Absolute,
            ..default()
        },
    ));

    commands.spawn((
        ChildOf(canvas),
        Text::new(CONSENT_NOTICE),
        TextFont {
            font: assets.pixel_font.clone(),
            font_size: 21.0,
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        Node {
            width: Val::Percent(85.0),
            bottom: Val::Px(50.0),
            position_type: PositionType::Absolute,
            ..default()
        },
    ));

    let button = commands
        .spawn((
            ChildOf(canvas),
            OkayButton,
            Button,
            ImageNode {
                image: Handle::<Image>::default(),
                color: GRAY_500.into(),
                ..default()
            },
            Node {
                width: Val::Px(150.0),
                height: Val::Px(40.0),
                bottom: Val::Px(-10.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    commands.spawn((
        ChildOf(button),
        Text::new("Okay"),
        TextFont {
            font: assets.pixel_font.clone(),
            font_size: 21.0,
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
    ));
}

fn handle_button(
    mut q_button: Query<(&mut ImageNode, &Interaction), With<OkayButton>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok((mut image, interaction)) = q_button.single_mut() else {
        return;
    };

    match interaction {
        Interaction::Pressed => next_state.set(GameState::Gaming),
        Interaction::Hovered => image.color = GRAY_700.into(),
        Interaction::None => image.color = GRAY_500.into(),
    }
}

fn despawn_consent(mut commands: Commands, q_button: Query<Entity, With<ConsentRoot>>) {
    for entity in &q_button {
        commands.entity(entity).despawn();
    }
}

fn check_consent(mut next_state: ResMut<NextState<GameState>>) {
    #[cfg(target_arch = "wasm32")]
    let has_consent = check_consent_wasm();

    #[cfg(not(target_arch = "wasm32"))]
    let has_consent = check_consent_native();

    if has_consent {
        next_state.set(GameState::Gaming);
    } else {
        next_state.set(GameState::ConsentNotice);
    }
}

#[cfg(target_arch = "wasm32")]
fn check_consent_wasm() -> bool {
    use crate::assets::WASM_CONSENT_STORAGE;
    use web_sys::window;

    let storage = window()
        .expect("failed to get window")
        .local_storage()
        .expect("failed to get local storage")
        .expect("failed to unwrap local storage");

    storage
        .get_item(WASM_CONSENT_STORAGE)
        .expect("failed to get local storage item WASM key")
        .is_some()
}

#[cfg(not(target_arch = "wasm32"))]
fn check_consent_native() -> bool {
    use crate::assets::CONSENT_FILE;
    use std::fs::read_to_string;

    !read_to_string(CONSENT_FILE)
        .expect("failed to read progression core file")
        .is_empty()
}

fn write_consent() {
    #[cfg(target_arch = "wasm32")]
    write_consent_wasm();

    #[cfg(not(target_arch = "wasm32"))]
    write_consent_native();
}

#[cfg(target_arch = "wasm32")]
fn write_consent_wasm() {
    use crate::assets::WASM_CONSENT_STORAGE;
    use web_sys::window;

    let storage = window()
        .expect("failed to get window")
        .local_storage()
        .expect("failed to get local storage")
        .expect("failed to unwrap local storage");

    storage
        .set_item(WASM_CONSENT_STORAGE, "You are awesome!")
        .expect("failed to write to consent storage");
}

#[cfg(not(target_arch = "wasm32"))]
fn write_consent_native() {
    use crate::assets::CONSENT_FILE;
    use std::fs;

    fs::write(CONSENT_FILE, "You are awesome!").expect("failed to write to consent file");
}

pub struct UiConsentPlugin;

impl Plugin for UiConsentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (check_consent).run_if(in_state(GameState::ConsentCheck)),
        )
        .add_systems(OnEnter(GameState::ConsentNotice), spawn_consent)
        .add_systems(
            OnExit(GameState::ConsentNotice),
            (write_consent, despawn_consent),
        )
        .add_systems(Update, handle_button);
    }
}
