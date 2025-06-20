use bevy::prelude::*;

use super::{CollisionGroups, StaticSensorAABB, StaticSensorCircle};

#[derive(Event)]
pub struct IntersectionEvent {
    pub entities: (Entity, Entity),
    pub collision_groups: (CollisionGroups, CollisionGroups),
}

fn relay_intersection_events_aabb_circle(
    q_aabbs: Query<(Entity, &Transform, &CollisionGroups, &StaticSensorAABB)>,
    q_circles: Query<
        (Entity, &Transform, &CollisionGroups, &StaticSensorCircle),
        Without<StaticSensorAABB>,
    >,
    mut ev_intersection: EventWriter<IntersectionEvent>,
) {
    for (entity_aabb, transform_aabb, groups_aabb, sensor_aabb) in &q_aabbs {
        for (entity_circle, transform_circle, groups_circle, sensor_circle) in &q_circles {
            if !groups_aabb.matches_with(groups_circle) {
                continue;
            }

            let circle_center = transform_circle.translation.xy() + sensor_circle.offset;
            let aabb_center = transform_aabb.translation.xy();
            let dis_sqrt = circle_center.distance_squared(aabb_center);

            if dis_sqrt > (sensor_aabb.outer_radius + sensor_circle.radius).powi(2) {
                continue;
            }

            if intersection_aabb_circle(
                sensor_circle.radius,
                circle_center,
                Vec2::new(sensor_aabb.half_x, sensor_aabb.half_y),
                aabb_center,
            ) {
                ev_intersection.write(IntersectionEvent {
                    entities: (entity_circle, entity_aabb),
                    collision_groups: (*groups_circle, *groups_aabb),
                });
            }
        }
    }
}

pub fn intersection_aabb_circle(
    radius: f32,
    circle_center: Vec2,
    k: Vec2,
    rect_center: Vec2,
) -> bool {
    let mut c = circle_center - rect_center;

    let mut sign = [1.0; 2];
    for i in 0..sign.len() {
        if c[i] < 0.0 {
            c[i] = -c[i];
            sign[i] = -1.0;
        }
    }

    do_query(k, c, radius)
}

fn do_query(k: Vec2, center: Vec2, radius: f32) -> bool {
    let delta = center - k;
    if delta.y > radius || delta.x > radius {
        return false;
    }

    if delta.y <= 0.0 || delta.x <= 0.0 {
        return true;
    }
    delta.length_squared() <= radius.powi(2)
}

pub struct IntersectionPlugin;

impl Plugin for IntersectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<IntersectionEvent>()
            .add_systems(PreUpdate, (relay_intersection_events_aabb_circle,).chain());
    }
}
