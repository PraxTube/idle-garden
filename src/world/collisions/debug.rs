use bevy::prelude::*;

use crate::world::DebugState;

use super::{ColliderColor, DynamicCollider, StaticCollider};

const DEFAULT_COLLIDER_COLOR: Color = Color::srgb(0.8, 0.3, 0.1);

pub fn visualize_colliders(
    mut gizmos: Gizmos,
    debug_state: Res<DebugState>,
    q_static_colliders: Query<(&GlobalTransform, &StaticCollider, Option<&ColliderColor>)>,
    q_dynamic_colliders: Query<
        (&Transform, &DynamicCollider, Option<&ColliderColor>),
        Without<StaticCollider>,
    >,
) {
    if !debug_state.active {
        return;
    }

    for (transform, collider, maybe_color) in &q_static_colliders {
        let color = match maybe_color {
            Some(color) => color.0,
            None => DEFAULT_COLLIDER_COLOR,
        };

        gizmos.rect_2d(
            Isometry2d::from_translation(transform.translation().xy()),
            2.0 * Vec2::new(collider.half_x, collider.half_y),
            color,
        );
    }

    for (transform, collider, maybe_color) in &q_dynamic_colliders {
        let color = match maybe_color {
            Some(color) => color.0,
            None => DEFAULT_COLLIDER_COLOR,
        };

        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.xy() + collider.offset()),
            collider.radius,
            color,
        );
    }
}
