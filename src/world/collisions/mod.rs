mod debug;
mod detection;
mod intersection;
mod response;
mod structs;

pub use intersection::{intersection_aabb_circle, IntersectionEvent};
pub use structs::*;

use bevy::{color::palettes::css::BLUE, prelude::*};
use response::collision_response;

use crate::GameState;

use super::{camera::CameraSystemSet, DebugState};

pub struct WorldCollisionPlugin;

impl Plugin for WorldCollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(intersection::IntersectionPlugin)
            .configure_sets(
                PostUpdate,
                CollisionSystemSet.before(CameraSystemSet::first()),
            )
            .add_systems(
                PostUpdate,
                (
                    update_velocities_with_collision.run_if(in_state(GameState::Gaming)),
                    update_transforms.run_if(in_state(GameState::Gaming)),
                    debug::visualize_colliders,
                    debug::visualize_sensors,
                )
                    .chain()
                    .in_set(CollisionSystemSet),
            );
    }
}

fn update_velocities_with_collision(
    mut gizmos: Gizmos,
    debug_state: Res<DebugState>,
    q_static_colliders: Query<(&GlobalTransform, &StaticCollider, &CollisionGroups)>,
    mut q_dynamic_colliders: Query<
        (
            Entity,
            &Transform,
            &mut Velocity,
            &DynamicCollider,
            &CollisionGroups,
        ),
        Without<StaticCollider>,
    >,
) {
    let mut new_velocities = Vec::new();
    for (actor_entity, circle_transform, velocity, circle, circle_collision_groups) in
        &q_dynamic_colliders
    {
        if velocity.0 == Vec2::ZERO {
            continue;
        }

        let circle_pos = circle_transform.translation.xy() + circle.offset;

        let mut rects = Vec::new();
        for (rect_transform, rect, collision_groups) in &q_static_colliders {
            if !collision_groups.matches_with(circle_collision_groups) {
                continue;
            }

            let rect_pos = rect_transform.translation().xy();
            rects.push((rect_pos, Vec2::new(rect.half_x, rect.half_y)));
        }

        let mut circle_colliders = Vec::new();
        for (collider_entity, circle_collider_transform, _, circle_collider, collision_groups) in
            &q_dynamic_colliders
        {
            if collider_entity == actor_entity {
                continue;
            }
            if !collision_groups.matches_with(circle_collision_groups) {
                continue;
            }

            let pos = circle_collider_transform.translation.xy() + circle_collider.offset;
            circle_colliders.push((pos, circle_collider.radius));
        }

        let (v, gizmo_lines) = collision_response(
            circle_pos,
            circle.radius,
            &rects,
            &circle_colliders,
            velocity.0,
        );
        new_velocities.push((actor_entity, v));

        if debug_state.active {
            for (start, end) in gizmo_lines {
                gizmos.line_2d(start, end, BLUE);
            }
        }
    }

    for (entity, v) in &new_velocities {
        let Ok((_, _, mut velocity, _, _)) = q_dynamic_colliders.get_mut(*entity) else {
            continue;
        };
        velocity.0 = *v;
    }
}

fn update_transforms(mut q_transforms: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut q_transforms {
        transform.translation += velocity.0.extend(0.0);
    }
}
