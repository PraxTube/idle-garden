use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::render::view::RenderLayers;

use crate::player::{GamingInput, Player};

use super::DebugState;

/// The amount of pixels that the game camera will span in the height.
const GAME_CAMERA_PROJECTION_SCALE: f32 = 250.0;
const PROJECTION_FAR: f32 = 1e6;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (zoom_camera,))
            .add_systems(
                PostUpdate,
                (apply_y_sort, apply_y_sort_child)
                    .chain()
                    .in_set(CameraSystemSet::ApplyYSort),
            )
            .add_systems(
                PostUpdate,
                update_camera_transform.in_set(CameraSystemSet::UpdateTransform),
            );
    }
}

/// Marker `Component` for the main camera.
/// There should only be one entity with this `Component`.
#[derive(Component)]
pub struct MainCamera;

/// Overwrites the z value of the Entities `Transform` Component
/// based on its y value.
#[derive(Component)]
pub struct YSort(pub f32);
/// Same as `YSort` but takes into account its parent `YSort`.
/// You will want to use this if the parent entity has a `YSort`.
///
/// For example, if you have a player and a player shadow than
/// you can use this for this shadow to have its own ysort.
#[derive(Component)]
pub struct YSortChild(pub f32);

/// Sets that are used to control the camera's transform.
/// They run before bevy's `TransformSystem::TransformPropagate`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CameraSystemSet {
    /// Set that will update the transform of the camera.
    /// You should not run anything in this set.
    UpdateTransform,
    /// Set that will update the z value based on the `YSort` component.
    /// You should not run anything in this set.
    ApplyYSort,
}

impl CameraSystemSet {
    /// The first SystemSet of the Camera SystemSets.
    /// If you need to run something before the camera system set, use this.
    pub fn first() -> Self {
        Self::UpdateTransform
    }

    /// The last SystemSet of the Camera SystemSets.
    /// If you need to run something after the camera system set, use this.
    #[allow(unused)]
    pub fn last() -> Self {
        Self::ApplyYSort
    }
}

fn apply_y_sort(mut q_transforms: Query<(&mut Transform, &YSort)>) {
    for (mut transform, ysort) in &mut q_transforms {
        transform.translation.z = ysort.0 - transform.translation.y;
    }
}

fn apply_y_sort_child(
    q_parents: Query<&Transform, (With<YSort>, Without<YSortChild>)>,
    mut q_transforms: Query<(&ChildOf, &mut Transform, &YSortChild), Without<YSort>>,
) {
    for (child_of, mut transform, ysort) in &mut q_transforms {
        let parent_transform = match q_parents.get(child_of.parent()) {
            Ok(r) => r,
            Err(_) => continue,
        };
        transform.translation.z =
            ysort.0 - transform.translation.y - parent_transform.translation.y;
    }
}

fn spawn_camera(mut commands: Commands) {
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: GAME_CAMERA_PROJECTION_SCALE,
        },
        far: PROJECTION_FAR,
        near: -PROJECTION_FAR,
        ..OrthographicProjection::default_2d()
    });

    commands.spawn((
        projection,
        MainCamera,
        Camera2d,
        RenderLayers::layer(0),
        IsDefaultUiCamera,
        Msaa::Off,
    ));
}

fn zoom_camera(
    debug_state: Res<DebugState>,
    gaming_input: Res<GamingInput>,
    mut q_projection: Query<&mut Projection, With<MainCamera>>,
) {
    if !debug_state.active {
        return;
    }
    if gaming_input.scroll == 0 {
        return;
    }

    let mut projection = match q_projection.single_mut() {
        Ok(p) => p,
        Err(_) => return,
    };

    let Projection::Orthographic(mut orth) = projection.clone() else {
        return;
    };

    orth.scale = (orth.scale + gaming_input.scroll as f32).clamp(1.0, 10.0);
    *projection = Projection::Orthographic(orth)
}

fn update_camera_transform(
    mut q_camera: Query<&mut Transform, With<MainCamera>>,
    q_player: Query<&Transform, (With<Player>, Without<MainCamera>)>,
) {
    let Ok(mut camera_transform) = q_camera.single_mut() else {
        return;
    };
    let Ok(player_transform) = q_player.single() else {
        return;
    };

    camera_transform.translation = player_transform.translation.with_z(0.0);
}
