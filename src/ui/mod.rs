mod store;

pub use store::ItemPressed;

use bevy::{prelude::*, window::WindowResized};

use crate::DEFAULT_WINDOW_WIDTH;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(store::UiStorePlugin)
            .add_systems(Update, scale_ui);
    }
}

fn scale_ui(mut ui_scale: ResMut<UiScale>, mut ev_window_resized: EventReader<WindowResized>) {
    for ev in ev_window_resized.read() {
        ui_scale.0 = ev.width / DEFAULT_WINDOW_WIDTH;
    }
}
