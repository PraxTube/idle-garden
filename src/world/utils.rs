use bevy::prelude::*;

/// Convert `Vec2` to `Quat` by taking angle between `Vec2::X`.
/// Returns `Quat::IDENTITY` for `Vec2::ZERO`.
pub fn quat_from_vec2(direction: Vec2) -> Quat {
    if direction == Vec2::ZERO {
        return Quat::IDENTITY;
    }
    Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, Vec2::X.angle_to(direction))
}
