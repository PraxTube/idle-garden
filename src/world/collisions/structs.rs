use bevy::prelude::*;

pub const WORLD_GROUP: u32 = 1;
pub const PLAYER_GROUP: u32 = 1 << 1;

/// Collision groups for colliders that are obstacles but don't block line of sight,
/// smaller objects essentially. (e.g. trashcans).
pub const WORLD_COLLISION_GROUPS: CollisionGroups =
    CollisionGroups::new(WORLD_GROUP, WORLD_GROUP | PLAYER_GROUP);
pub const PLAYER_COLLISION_GROUPS: CollisionGroups =
    CollisionGroups::new(PLAYER_GROUP, WORLD_GROUP);

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

    pub fn offset(&self) -> Vec2 {
        self.offset
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
    pub fn interacts_with(&self, other: &Self) -> bool {
        self.memberships & other.filters != 0 && self.filters & other.memberships != 0
    }
}

impl StaticCollider {
    pub fn new(half_x: f32, half_y: f32) -> Self {
        Self { half_x, half_y }
    }
}
