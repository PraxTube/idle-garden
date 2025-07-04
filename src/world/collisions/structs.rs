use bevy::prelude::*;

pub const WORLD_GROUP: u32 = 1;
pub const PLAYER_GROUP: u32 = 1 << 1;
pub const GRASS_GROUP: u32 = 1 << 2;
pub const SLASH_GROUP: u32 = 1 << 3;

pub const WORLD_COLLISION_GROUPS: CollisionGroups =
    CollisionGroups::new(WORLD_GROUP, WORLD_GROUP | PLAYER_GROUP);
pub const PLAYER_COLLISION_GROUPS: CollisionGroups =
    CollisionGroups::new(PLAYER_GROUP, WORLD_GROUP);
pub const GRASS_COLLISION_GROUPS: CollisionGroups = CollisionGroups::new(GRASS_GROUP, SLASH_GROUP);
pub const SLASH_COLLISION_GROUPS: CollisionGroups = CollisionGroups::new(SLASH_GROUP, GRASS_GROUP);

/// Sets that are used to control the camera's transform.
/// Runs before the whole CameraSystemSet.
/// Note: It is expected that the CameraSystemSet runs before bevy's Transform propagate system
/// set.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollisionSystemSet;

#[derive(Component)]
pub struct DynamicCollider {
    pub radius: f32,
    pub offset: Vec2,
}

#[derive(Component)]
pub struct StaticCollider {
    pub half_x: f32,
    pub half_y: f32,
}

#[derive(Component)]
#[require(Transform)]
pub struct StaticSensorAABB {
    pub half_x: f32,
    pub half_y: f32,
    pub outer_radius: f32,
}

#[derive(Component)]
pub struct StaticSensorCircle {
    pub radius: f32,
    pub offset: Vec2,
}

#[derive(Component, Clone, Copy, PartialEq)]
pub struct CollisionGroups {
    pub memberships: u32,
    pub filters: u32,
}

#[derive(Component)]
pub struct ColliderColor(pub Color);

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

/// Marker component for world collisions.
/// These are all colliders that have World group as their membership.
/// E.g wall colliders, props or border colliders.
#[derive(Component)]
pub struct WorldCollider;

impl DynamicCollider {
    pub fn new(radius: f32, offset: Vec2) -> Self {
        Self { radius, offset }
    }
}

impl CollisionGroups {
    pub const fn new(memberships: u32, filters: u32) -> Self {
        Self {
            memberships,
            filters,
        }
    }

    /// Whether the current and the other CollisionGroups are on compatible layers and can interact
    /// (collision/intersections can occure).
    pub fn matches_with(&self, other: &Self) -> bool {
        self.memberships & other.filters != 0 && self.filters & other.memberships != 0
    }
}

impl StaticCollider {
    pub fn new(half_x: f32, half_y: f32) -> Self {
        Self { half_x, half_y }
    }
}

impl StaticSensorAABB {
    pub fn new(half_x: f32, half_y: f32) -> Self {
        let outer_radius = (half_x.powi(2) + half_y.powi(2)).sqrt();
        Self {
            half_x,
            half_y,
            outer_radius,
        }
    }
}

impl StaticSensorCircle {
    pub fn new(radius: f32, offset: Vec2) -> Self {
        Self { radius, offset }
    }
}
