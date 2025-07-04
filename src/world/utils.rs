use bevy::prelude::*;

/// Convert `Vec2` to `Quat` by taking angle between `Vec2::X`.
/// Returns `Quat::IDENTITY` for `Vec2::ZERO`.
pub fn quat_from_vec2(direction: Vec2) -> Quat {
    if direction == Vec2::ZERO {
        return Quat::IDENTITY;
    }
    Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, Vec2::X.angle_to(direction))
}

pub fn format_money_string_raw(amount: u64) -> String {
    if amount < 10_000 {
        format!("{}", amount)
    } else if amount < 1_000_000 {
        let thousands = amount / 1_000;
        debug_assert!(thousands > 9);
        debug_assert!(thousands < 1_000);

        if thousands < 100 {
            let remainder = (amount % 1_000) / 100;
            format!("{}.{}k", thousands, remainder)
        } else {
            format!("{}k", thousands)
        }
    } else if amount < 1_000_000_000 {
        let millions = amount / 1_000_000;
        debug_assert!(millions > 0);

        if millions < 10 {
            let remainder = (amount % 1_000_000) / 10_000;
            format!("{}.{}M", millions, remainder)
        } else if millions < 100 {
            let remainder = (amount % 1_000_000) / 100_000;
            format!("{}.{}M", millions, remainder)
        } else {
            format!("{}M", millions)
        }
    } else {
        let billions = amount / 1_000_000_000;
        debug_assert!(billions > 0);

        if billions < 10 {
            let remainder = (amount % 1_000_000_000) / 10_000_000;
            format!("{}.{}B", billions, remainder)
        } else if billions < 100 {
            let remainder = (amount % 1_000_000_000) / 100_000_000;
            format!("{}.{}B", billions, remainder)
        } else {
            format!("{}B", billions)
        }
    }
}

pub fn format_money_string(amount: u64) -> String {
    "$".to_string() + &format_money_string_raw(amount)
}

#[test]
fn validate_money_format_string() {
    assert_eq!(format_money_string_raw(24_300_001), "24.3M".to_string());
    assert_eq!(format_money_string(24_300_001), "$24.3M".to_string());
    assert_eq!(format_money_string(24_000_300_001), "$24.0B".to_string());
    assert_eq!(
        format_money_string(24_000_000_300_001),
        "$24000B".to_string()
    );
}
