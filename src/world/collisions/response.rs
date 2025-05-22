use bevy::prelude::*;

use super::detection::collision_contact_data;

const MAX_ITERATIONS: usize = 4;
/// Squared length of velocity vector under which the collision response will terminate early.
const SQRT_VELOCITY_MAGNITUDE_THRESHOLD: f32 = 0.1;
/// The mininum fraction of the magnitude of the velocity vector.
/// If the output velocity (squared) length is smaller then this value times the start velocity
/// (squared) length, then clamp to ZERO vector.
const MIN_VELOCITY_FRACTION_THRESHOLD: f32 = 0.01;
/// Amount that we push dynamic collider outside of any other colliders (add to velocity vector).
const NORMAL_VECTOR_PUSHBACK_RATIO: f32 = 0.1;

pub fn collision_response(
    start_c: Vec2,
    radius: f32,
    rects: &[(Vec2, Vec2)],
    circles: &[(Vec2, f32)],
    start_velocity: Vec2,
) -> (Vec2, Vec<(Vec2, Vec2)>) {
    let (v, gizmo_lines) =
        get_potentially_faulty_velocity(start_c, radius, rects, circles, start_velocity);

    if v == Vec2::ZERO
        || v.length_squared() < start_velocity.length_squared() * MIN_VELOCITY_FRACTION_THRESHOLD
        || is_colliding_with_any_rect(start_c, radius, rects, v)
    {
        (Vec2::ZERO, gizmo_lines)
    } else {
        (v, gizmo_lines)
    }
}

fn get_potentially_faulty_velocity(
    start_c: Vec2,
    radius: f32,
    rects: &[(Vec2, Vec2)],
    circles: &[(Vec2, f32)],
    start_velocity: Vec2,
) -> (Vec2, Vec<(Vec2, Vec2)>) {
    let mut safe_velocity;
    let mut safe_c;

    let mut velocity = start_velocity;
    let mut c = start_c;
    let mut gizmo_lines = Vec::new();

    for _ in 0..MAX_ITERATIONS {
        if velocity.length_squared() < SQRT_VELOCITY_MAGNITUDE_THRESHOLD {
            break;
        }

        // if i == MAX_ITERATIONS - 1 {
        //     warn!("reached max iters!!!!!!!!!!!!!!!!, {}", velocity);
        // }

        let mut collisions = Vec::new();
        for (rect_pos, k) in rects {
            let contact_data = collision_contact_data(radius, c, *k, *rect_pos, velocity);

            if let Some((t, p, n)) = contact_data {
                collisions.push((t, p, n, rect_pos));
            }
        }

        for (circle_pos, circle_collider_radius) in circles {
            let contact_data = collision_contact_data(
                radius + *circle_collider_radius,
                c,
                Vec2::ZERO,
                *circle_pos,
                velocity,
            );

            if let Some((t, p, n)) = contact_data {
                collisions.push((t, p, n, circle_pos));
            }
        }

        let Some((t, _, n, rect_pos)) = collisions.iter().min_by(|a, b| a.0.total_cmp(&b.0)) else {
            break;
        };

        safe_velocity = velocity * t;
        safe_c = c;

        let (new_pos_delta, new_v) = slide(velocity, *n, *t);
        c += new_pos_delta;
        velocity = new_v;

        gizmo_lines.push((c, **rect_pos));

        for (rect_pos, k) in rects {
            if is_slide_colliding(radius, c, *k, *rect_pos, velocity) {
                return (safe_c - start_c + safe_velocity, gizmo_lines);
            }
        }
    }
    (c - start_c + velocity, gizmo_lines)
}

fn slide(v: Vec2, n: Vec2, t: f32) -> (Vec2, Vec2) {
    if v.dot(n) > 0.0 {
        return (v, Vec2::ZERO);
    }
    // Push slightly out of the current collider.
    // The norm will always point away from the collider point.
    let new_v = v * t + n * NORMAL_VECTOR_PUSHBACK_RATIO;
    let remaining_v = v - new_v;

    if remaining_v.length_squared() < SQRT_VELOCITY_MAGNITUDE_THRESHOLD {
        return (new_v, Vec2::ZERO);
    }

    let plane = if v.dot(n.perp()) < 0.0 {
        -n.perp()
    } else {
        n.perp()
    };

    (new_v, plane * plane.dot(remaining_v))
}

fn is_slide_colliding(
    radius: f32,
    circle_center: Vec2,
    k: Vec2,
    rect_center: Vec2,
    v: Vec2,
) -> bool {
    let Some((t, _, n)) = collision_contact_data(radius, circle_center, k, rect_center, v) else {
        return false;
    };

    if t > 0.0 {
        return false;
    }

    v.dot(n) < 0.0
}

fn is_colliding_with_any_rect(
    c: Vec2,
    radius: f32,
    rects: &[(Vec2, Vec2)],
    velocity: Vec2,
) -> bool {
    for (rect_pos, k) in rects {
        let Some((t, _, _)) = collision_contact_data(radius, c, *k, *rect_pos, velocity) else {
            continue;
        };

        if t > 0.0 {
            continue;
        }

        return true;
    }
    false
}
