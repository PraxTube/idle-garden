use bevy::prelude::*;
use bevy_trickfilm::{animation::event::EventTarget, prelude::*};
use bevy_trickfilm_derive::AnimationEvent;

#[derive(Debug, Clone, Event, Reflect, AnimationEvent)]
pub struct PlayerFootstepEvent {
    #[reflect(skip_serializing)]
    #[animationevent(target)]
    target: EventTarget,
    right_foot: bool,
}

pub struct GameAssetsEventsPlugin;

impl Plugin for GameAssetsEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_animation_event::<PlayerFootstepEvent>();
    }
}
