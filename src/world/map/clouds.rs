use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::*,
    sprite::{AlphaMode2d, Material2d, Material2dPlugin},
};

use crate::{assets::CLOUDS_SHADER, world::MainCamera, GameAssets, GameState};

use super::ZLevel;

const CLOUDS_NOISE_WIDTH: f32 = 512.0;
const CLOUDS_NOISE_HEIGHT: f32 = 256.0;

#[derive(Component)]
enum Cloud {
    TopRight,
    BottomRight,
    BottomLeft,
    TopLeft,
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
struct CloudsMaterial {
    #[texture(0)]
    #[sampler(1)]
    primary: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    secondary: Handle<Image>,
    #[texture(4)]
    #[sampler(5)]
    tertiary: Handle<Image>,
    #[texture(6)]
    #[sampler(7)]
    quaternary: Handle<Image>,
    #[uniform(8)]
    texel_size: Vec4,
}

impl Cloud {
    fn offset(&self) -> Vec2 {
        match self {
            Cloud::TopRight => Vec2::new(CLOUDS_NOISE_WIDTH * 0.5, CLOUDS_NOISE_HEIGHT * 0.5),
            Cloud::BottomRight => Vec2::new(CLOUDS_NOISE_WIDTH * 0.5, -CLOUDS_NOISE_HEIGHT * 0.5),
            Cloud::BottomLeft => Vec2::new(-CLOUDS_NOISE_WIDTH * 0.5, -CLOUDS_NOISE_HEIGHT * 0.5),
            Cloud::TopLeft => Vec2::new(-CLOUDS_NOISE_WIDTH * 0.5, CLOUDS_NOISE_HEIGHT * 0.5),
        }
    }
}

impl Material2d for CloudsMaterial {
    fn fragment_shader() -> ShaderRef {
        CLOUDS_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

fn spawn_clouds(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CloudsMaterial>>,
) {
    let transform = Transform::from_xyz(0.0, 0.0, ZLevel::TopEnvironment.value())
        .with_scale(Vec3::new(CLOUDS_NOISE_WIDTH, CLOUDS_NOISE_HEIGHT, 1.0));
    let mesh = Mesh2d(meshes.add(Rectangle::default()));
    let mat = MeshMaterial2d(materials.add(CloudsMaterial {
        primary: assets.primary_clouds_noise_texture.clone(),
        secondary: assets.secondary_clouds_noise_texture.clone(),
        tertiary: assets.tertiary_clouds_noise_texture.clone(),
        quaternary: assets.quaternary_clouds_noise_texture.clone(),
        texel_size: Vec4::new(
            1.0 / CLOUDS_NOISE_WIDTH,
            1.0 / CLOUDS_NOISE_HEIGHT,
            0.0,
            0.0,
        ),
    }));

    for cloud in [
        Cloud::TopRight,
        Cloud::BottomRight,
        Cloud::BottomLeft,
        Cloud::TopLeft,
    ] {
        commands.spawn((cloud, transform.clone(), mesh.clone(), mat.clone()));
    }
}

fn reposition_clouds(
    q_camera: Query<&Transform, With<MainCamera>>,
    mut q_clouds: Query<(&mut Transform, &Cloud), Without<MainCamera>>,
) {
    let Ok(camera_transform) = q_camera.single() else {
        return;
    };

    let snapped_pos = Vec2::new(
        (camera_transform.translation.x / CLOUDS_NOISE_WIDTH).round() * CLOUDS_NOISE_WIDTH,
        (camera_transform.translation.y / CLOUDS_NOISE_HEIGHT).round() * CLOUDS_NOISE_HEIGHT,
    );

    for (mut cloud_transform, cloud) in &mut q_clouds {
        let pos = snapped_pos + cloud.offset();
        cloud_transform.translation.x = pos.x;
        cloud_transform.translation.y = pos.y;
    }
}

#[cfg(debug_assertions)]
fn validate_const_match(assets: Res<GameAssets>, images: Res<Assets<Image>>) {
    let primary_image = images.get(&assets.primary_clouds_noise_texture).unwrap();
    assert_eq!(primary_image.width() as f32, CLOUDS_NOISE_WIDTH);
    assert_eq!(primary_image.height() as f32, CLOUDS_NOISE_HEIGHT);

    let secondary_image = images.get(&assets.secondary_clouds_noise_texture).unwrap();
    assert_eq!(secondary_image.size(), primary_image.size());
    let tertiary_image = images.get(&assets.tertiary_clouds_noise_texture).unwrap();
    assert_eq!(tertiary_image.size(), primary_image.size());
}

pub struct CloudsPlugin;

impl Plugin for CloudsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<CloudsMaterial>::default())
            .add_systems(OnExit(GameState::AssetLoading), spawn_clouds)
            .add_systems(Update, reposition_clouds);

        #[cfg(debug_assertions)]
        app.add_systems(OnExit(GameState::AssetLoading), validate_const_match);
    }
}
