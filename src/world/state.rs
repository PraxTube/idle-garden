use bevy::prelude::*;

use crate::{
    player::GamingInput,
    ui::{MenuAction, MenuActionEvent},
    GameState,
};

fn switch_to_menu_state(
    mut next_state: ResMut<NextState<GameState>>,
    gaming_input: Res<GamingInput>,
) {
    if !gaming_input.menu {
        return;
    }
    next_state.set(GameState::Menu);
}

fn switch_to_gaming_state_from_menu(
    mut next_state: ResMut<NextState<GameState>>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    if ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::Continue || ev.action == MenuAction::Reset)
    {
        next_state.set(GameState::Gaming);
    }
}

pub struct WorldStatePlugin;

impl Plugin for WorldStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                switch_to_menu_state.run_if(in_state(GameState::Gaming)),
                switch_to_gaming_state_from_menu.run_if(in_state(GameState::Menu)),
            ),
        );
    }
}
