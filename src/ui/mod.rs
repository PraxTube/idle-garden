mod auto_save_icon;
mod consent;
mod debug;
mod menu;
mod outline;
mod stats;
mod store;

pub use menu::{MenuAction, MenuActionEvent};
pub use store::ItemPressed;

use bevy::{prelude::*, window::WindowResized};

use crate::DEFAULT_WINDOW_WIDTH;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            debug::UiDebugPlugin,
            outline::UiOutlinePlugin,
            consent::UiConsentPlugin,
            auto_save_icon::UiAutoSaveIconPlugin,
            stats::UiStatsPlugin,
            store::UiStorePlugin,
            menu::UiMenuPlugin,
        ))
        .add_systems(Update, scale_ui);
    }
}

fn scale_ui(mut ui_scale: ResMut<UiScale>, mut ev_window_resized: EventReader<WindowResized>) {
    for ev in ev_window_resized.read() {
        ui_scale.0 = ev.width / DEFAULT_WINDOW_WIDTH;
    }
}
