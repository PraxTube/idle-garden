use std::f32::consts::FRAC_1_SQRT_2;

use bevy::prelude::*;

pub fn collision_contact_data(
    radius: f32,
    circle_center: Vec2,
    k: Vec2,
    rect_center: Vec2,
    mut v: Vec2,
) -> Option<(f32, Vec2, Vec2)> {
    let mut c = circle_center - rect_center;

    let mut sign = [1.0; 2];
    for i in 0..sign.len() {
        if c[i] < 0.0 {
            c[i] = -c[i];
            v[i] = -v[i];
            sign[i] = -1.0;
        }
    }

    let (t, mut p, mut n) = do_query(k, c, radius, v)?;

    if t > 1.0 {
        return None;
    }

    for i in 0..sign.len() {
        p[i] = rect_center[i] + sign[i] * p[i];
        n[i] *= sign[i]
    }

    Some((t, p, n))
}

fn perp(v: Vec2) -> Vec2 {
    Vec2::new(v.y, -v.x)
}

fn intersects_arc(
    k: Vec2,
    q0: f32,
    q1: f32,
    q2: f32,
    c: Vec2,
    v: Vec2,
) -> Option<(f32, Vec2, Vec2)> {
    let t = (q0 - q1.sqrt()) / q2;
    let n = c + v * t - k;
    Some((t, k, n.normalize_or_zero()))
}

fn intersects_right_edge(k: Vec2, c: Vec2, radius: f32, v: Vec2) -> Option<(f32, Vec2, Vec2)> {
    let t = (k.x + radius - c.x) / v.x;
    Some((t, Vec2::new(k.x, c.y + t * v.y), Vec2::X))
}

fn intersects_top_edge(k: Vec2, c: Vec2, radius: f32, v: Vec2) -> Option<(f32, Vec2, Vec2)> {
    let t = (k.y + radius - c.y) / v.y;
    Some((t, Vec2::new(c.x + t * v.x, k.y), Vec2::Y))
}

fn do_query(k: Vec2, center: Vec2, radius: f32, v: Vec2) -> Option<(f32, Vec2, Vec2)> {
    let delta = center - k;
    if delta.y <= radius {
        if delta.x <= radius {
            if delta.y <= 0.0 {
                if delta.x <= 0.0 {
                    in_region_0(center)
                } else {
                    in_region_1(k, center, delta, radius, v)
                }
            } else if delta.x <= 0.0 {
                in_region_2(k, center, delta, radius, v)
            } else if delta.length_squared() <= radius.powi(2) {
                in_region_3(k, delta, radius, v)
            } else {
                in_region_4(k, center, delta, radius, v)
            }
        } else {
            in_region_5(k, center, delta, radius, v)
        }
    } else if delta.x <= radius {
        in_region_6(k, center, delta, radius, v)
    } else {
        in_region_7(k, center, delta, radius, v)
    }
}

fn in_region_0(c: Vec2) -> Option<(f32, Vec2, Vec2)> {
    Some((0.0, c, Vec2::ONE * FRAC_1_SQRT_2))
}

fn in_region_1(k: Vec2, c: Vec2, _delta: Vec2, _radius: f32, v: Vec2) -> Option<(f32, Vec2, Vec2)> {
    if v.x >= 0.0 {
        return None;
    }
    Some((0.0, Vec2::new(k.x, c.y), Vec2::X))
}

fn in_region_2(k: Vec2, c: Vec2, _delta: Vec2, _radius: f32, v: Vec2) -> Option<(f32, Vec2, Vec2)> {
    if v.y >= 0.0 {
        return None;
    }
    Some((0.0, Vec2::new(c.x, k.y), Vec2::Y))
}

fn in_region_3(k: Vec2, _delta: Vec2, _radius: f32, v: Vec2) -> Option<(f32, Vec2, Vec2)> {
    if v.x >= 0.0 && v.y >= 0.0 {
        return None;
    }
    Some((0.0, k, Vec2::ONE * FRAC_1_SQRT_2))
}

fn in_region_4(k: Vec2, c: Vec2, delta: Vec2, radius: f32, v: Vec2) -> Option<(f32, Vec2, Vec2)> {
    let neg_dot_v_del = -v.dot(delta);

    if neg_dot_v_del <= 0.0 {
        return None;
    }

    let dot_v = v.dot(v);
    let dot_v_perp_del = v.dot(perp(delta));
    let discr = radius.powi(2) * dot_v - dot_v_perp_del.powi(2);
    if discr < 0.0 {
        return None;
    }
    intersects_arc(k, neg_dot_v_del, discr, dot_v, c, v)
}

fn in_region_5(k0: Vec2, c: Vec2, delta0: Vec2, radius: f32, v: Vec2) -> Option<(f32, Vec2, Vec2)> {
    if v.x >= 0.0 {
        return None;
    }

    let dot_v_perp_del0 = v.dot(perp(delta0));
    let dot_v_perp_u0 = -radius * v.y - dot_v_perp_del0;

    if dot_v_perp_u0 <= 0.0 {
        let k1 = Vec2::new(k0.x, -k0.y);
        let delta1 = c - k1;
        let dot_v_perp_del1 = v.dot(perp(delta1));
        let dot_v_perp_u1 = -radius * v.y - dot_v_perp_del1;
        if dot_v_perp_u1 >= 0.0 {
            return intersects_right_edge(k0, c, radius, v);
        }

        let dot_v = v.dot(v);
        let discr = radius.powi(2) * dot_v - dot_v_perp_del1.powi(2);
        if discr < 0.0 {
            return None;
        }
        intersects_arc(k1, -v.dot(delta1), discr, dot_v, c, v)
    } else {
        let dot_v = v.dot(v);
        let discr = radius.powi(2) * dot_v - dot_v_perp_del0.powi(2);
        if discr < 0.0 {
            return None;
        }
        intersects_arc(k0, -v.dot(delta0), discr, dot_v, c, v)
    }
}

fn in_region_6(k0: Vec2, c: Vec2, delta0: Vec2, radius: f32, v: Vec2) -> Option<(f32, Vec2, Vec2)> {
    if v.y >= 0.0 {
        return None;
    }

    let dot_v_perp_del0 = v.dot(perp(delta0));
    let dot_v_perp_w0 = radius * v.x - dot_v_perp_del0;

    if dot_v_perp_w0 >= 0.0 {
        let k2 = Vec2::new(-k0.x, k0.y);
        let delta2 = c - k2;
        let dot_v_perp_del2 = v.dot(perp(delta2));
        let dot_v_perp_w2 = radius * v.x - dot_v_perp_del2;
        if dot_v_perp_w2 <= 0.0 {
            return intersects_top_edge(k0, c, radius, v);
        }

        let dot_v = v.dot(v);
        let discr = radius.powi(2) * dot_v - dot_v_perp_del2.powi(2);
        if discr < 0.0 {
            return None;
        }
        intersects_arc(k2, -v.dot(delta2), discr, dot_v, c, v)
    } else {
        let dot_v = v.dot(v);
        let discr = radius.powi(2) * dot_v - dot_v_perp_del0.powi(2);
        if discr < 0.0 {
            return None;
        }
        intersects_arc(k0, -v.dot(delta0), discr, dot_v, c, v)
    }
}

fn in_region_7(k0: Vec2, c: Vec2, delta0: Vec2, radius: f32, v: Vec2) -> Option<(f32, Vec2, Vec2)> {
    if v.x >= 0.0 || v.y >= 0.0 {
        return None;
    }

    let dot_v_perp_del0 = v.dot(perp(delta0));
    let dot_v_perp_w0 = radius * v.x - dot_v_perp_del0;
    if dot_v_perp_w0 <= 0.0 {
        let dot_v_perp_u0 = -radius * v.y - dot_v_perp_del0;
        if dot_v_perp_u0 >= 0.0 {
            let dot_v = v.dot(v);
            let discr = radius.powi(2) * dot_v - dot_v_perp_del0.powi(2);
            intersects_arc(k0, -v.dot(delta0), discr, dot_v, c, v)
        } else {
            let k1 = Vec2::new(k0.x, -k0.y);
            let delta1 = c - k1;
            let dot_v_perp_del1 = v.dot(perp(delta1));
            let dot_v_perp_u1 = -radius * v.y - dot_v_perp_del1;
            if dot_v_perp_u1 >= 0.0 {
                intersects_right_edge(k0, c, radius, v)
            } else {
                let dot_v = v.dot(v);
                let discr = radius.powi(2) * dot_v - dot_v_perp_del1.powi(2);
                if discr < 0.0 {
                    return None;
                }
                intersects_arc(k1, -v.dot(delta1), discr, dot_v, c, v)
            }
        }
    } else {
        let k2 = Vec2::new(-k0.x, k0.y);
        let delta2 = c - k2;
        let dot_v_perp_del2 = v.dot(perp(delta2));
        let dot_v_perp_w2 = radius * v.x - dot_v_perp_del2;
        if dot_v_perp_w2 <= 0.0 {
            intersects_top_edge(k0, c, radius, v)
        } else {
            let dot_v = v.dot(v);
            let discr = radius.powi(2) * dot_v - dot_v_perp_del2.powi(2);
            if discr < 0.0 {
                return None;
            }
            intersects_arc(k2, -v.dot(delta2), discr, dot_v, c, v)
        }
    }
}

#[test]
fn moving_region_4() {
    let k = Vec2::new(4.0, 4.0);
    let radius = 16.0;

    let circle_center = Vec2::new(119.0, 119.0);
    let rect_center = Vec2::new(100.0, 100.0);

    for (v, expected) in [
        (Vec2::new(-10.0, -10.0), Some(0.0)),
        (Vec2::new(-1.0, -1.0), None),
        (Vec2::new(1.0, -1.0), None),
        (Vec2::new(-100.0, 100.0), None),
    ] {
        let maybe_t_and_p = collision_contact_data(radius, circle_center, k, rect_center, v);
        match expected {
            Some(_) => assert!(maybe_t_and_p.is_some()),
            None => assert!(maybe_t_and_p.is_none(), "{}", v),
        }
    }
}

#[test]
fn fast_moving_region_5() {
    let k = Vec2::new(5.0, 24.0);
    let radius = 16.0;

    let circle_center = Vec2::new(100.0, 100.0);
    let rect_center = Vec2::new(300.0, 100.0);

    for (v, expected) in [
        (Vec2::new(500.0, 0.0), Some(0.0)),
        (Vec2::new(100.0, 0.0), None),
    ] {
        let maybe_t_and_p = collision_contact_data(radius, circle_center, k, rect_center, v);
        match expected {
            Some(_) => assert!(maybe_t_and_p.is_some()),
            None => assert!(maybe_t_and_p.is_none()),
        }
    }
}

#[test]
fn fast_moving_region_6() {
    let k = Vec2::new(24.0, 5.0);
    let radius = 16.0;

    let circle_center = Vec2::new(100.0, 100.0);
    let rect_center = Vec2::new(100.0, 300.0);

    for (v, expected) in [
        (Vec2::new(0.0, 500.0), Some(0.0)),
        (Vec2::new(0.0, 100.0), None),
    ] {
        let maybe_t_and_p = collision_contact_data(radius, circle_center, k, rect_center, v);
        match expected {
            Some(_) => assert!(maybe_t_and_p.is_some()),
            None => assert!(maybe_t_and_p.is_none()),
        }
    }
}

#[test]
fn fast_moving_region_7() {
    let k = Vec2::new(24.0, 5.0);
    let radius = 16.0;

    let circle_center = Vec2::new(100.0, 100.0);
    let rect_center = Vec2::new(300.0, 300.0);

    for (v, expected) in [
        (Vec2::new(500.0, 500.0), Some(0.0)),
        (Vec2::new(100.0, 100.0), None),
    ] {
        let maybe_t_and_p = collision_contact_data(radius, circle_center, k, rect_center, v);
        match expected {
            Some(_) => assert!(maybe_t_and_p.is_some()),
            None => assert!(maybe_t_and_p.is_none()),
        }
    }
}

#[test]
fn validate_contact_point() {
    let distance = 100.0;

    let radius = 0.0;
    let c = Vec2::ZERO;
    let k = Vec2::ONE * 12.0;
    let r = Vec2::Y * distance;
    let dir = Vec2::Y;

    assert!(collision_contact_data(radius, c, k, r, dir).is_none());
    assert!(collision_contact_data(radius, c, k, r, dir * (distance - 0.1 - radius) - k).is_none());
    assert!(collision_contact_data(radius, c, k, r, dir * (distance - radius) - k).is_some());

    let (t, p, n) = collision_contact_data(radius, c, k, r, dir * (distance - radius) - k).unwrap();

    assert!((t - 1.0).abs() < 1e-4);
    assert_eq!(p, Vec2::Y * distance - k - Vec2::Y * radius);
    assert_eq!(n, Vec2::NEG_Y);
}

#[test]
fn validate_toi_is_clamped() {
    let radius = 0.0;
    let c = Vec2::ZERO;
    let k = Vec2::ONE * 12.0;
    let r = Vec2::ONE * 100.0;
    let dir = Vec2::ONE.normalize();

    let distance = (r - c).length();

    assert!(dir.length() - 1.0 < 1e-06);

    assert!(collision_contact_data(radius, c, k, r, dir).is_none());
    assert!(collision_contact_data(radius, c, k, r, dir * 10.0).is_none());
    assert!(collision_contact_data(radius, c, k, r, dir * (distance - 0.1 - radius) - k).is_none());
    assert!(collision_contact_data(radius, c, k, r, dir * (distance - radius) - k).is_some());

    assert!(dir.length() - 1.0 < 1e-06);

    let (t, p, _) = collision_contact_data(radius, c, k, r, dir * (distance - radius) - k).unwrap();

    assert!(t > 0.0);
    assert!(t - 1.0 < 1e-4);
    assert_eq!(p, r - radius - k);

    assert!(
        collision_contact_data(radius, c, k, r, dir * distance)
            .unwrap()
            .0
            < 1.0
    );
}
