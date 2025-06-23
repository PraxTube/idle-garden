use bevy::{prelude::*, text::FontSmoothing};

use crate::{world::AutoSave, GameAssets};

use super::outline::TextOutline;

const AUTO_SAVE_TEXTS: [&str; 4] = ["Saving", "Saving.", "Saving..", "Saving..."];

#[derive(Component)]
struct AutoSaveIcon {
    timer: Timer,
    delete_timer: Timer,
    current_index: usize,
}

fn spawn_auto_save_icon(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        AutoSaveIcon {
            timer: Timer::from_seconds(0.35, TimerMode::Repeating),
            delete_timer: Timer::from_seconds(3.0, TimerMode::Once),
            current_index: 0,
        },
        TextOutline::new(
            "Saving...".to_string(),
            1.0,
            Color::WHITE,
            Color::BLACK,
            TextFont {
                font: assets.pixel_font.clone(),
                font_size: 20.0,
                font_smoothing: FontSmoothing::None,
                ..default()
            },
            false,
        ),
        Node {
            left: Val::Percent(80.0),
            top: Val::Px(80.0),
            position_type: PositionType::Absolute,
            ..default()
        },
    ));
}

fn animate_auto_save_icon(
    time: Res<Time>,
    mut q_auto_save_text: Query<(&mut TextOutline, &mut AutoSaveIcon)>,
) {
    let Ok((mut outline, mut auto_save)) = q_auto_save_text.single_mut() else {
        return;
    };

    auto_save.timer.tick(time.delta());
    if !auto_save.timer.just_finished() {
        return;
    };

    auto_save.current_index = (auto_save.current_index + 1) % AUTO_SAVE_TEXTS.len();
    outline.text = AUTO_SAVE_TEXTS[auto_save.current_index].to_string();
}

fn despawn_auto_save_icon(
    mut commands: Commands,
    time: Res<Time>,
    mut q_auto_save_icon: Query<(Entity, &mut AutoSaveIcon)>,
) {
    for (entity, mut auto_save) in &mut q_auto_save_icon {
        auto_save.delete_timer.tick(time.delta());
        if auto_save.delete_timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub struct UiAutoSaveIconPlugin;

impl Plugin for UiAutoSaveIconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                despawn_auto_save_icon,
                spawn_auto_save_icon.run_if(on_event::<AutoSave>),
                animate_auto_save_icon,
            )
                .chain(),
        );
    }
}
