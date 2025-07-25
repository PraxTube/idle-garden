use bevy::{
    color::palettes::css::RED,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin},
    text::FontSmoothing,
};
use rand::{thread_rng, Rng};

use crate::{
    assets::GRASS_SHADER,
    player::Player,
    ui::{MenuAction, MenuActionEvent},
    world::{
        collisions::{IntersectionEvent, StaticSensorAABB, GRASS_COLLISION_GROUPS},
        utils::format_money_string_raw,
        DynamicCollider, Velocity, YSort, ZLevel, SLASH_COLLISION_GROUPS, TILE_SIZE,
    },
    GameState,
};

use crate::GameAssets;

use super::{
    flora::InitialFloraSpawned, ItemBought, MapData, ProgressionCore, ProgressionSystemSet,
    MAP_SIZE, TALL_GRASS_CELL_VALUE,
};

// Should match the exp damp time scale used in the grass shader.
// The sine time will only be reset when the exp damp is at zero,
// in other words when the grass is not moving through player shake.
const TIME_TILL_SINE_RESET: f32 = 1.5;
const OFFLINE_PROGRESSION_NUMBER_POP_UP_OFFSET: Vec2 = Vec2::new(0.0, 20.0);
const CUT_TALL_GRASS_POINTS: u64 = 1;

const QUAD_MAX_SHIFT_OFFSET: f32 = 3.0;
const QUAD_OFFSETS: [Vec2; 4] = [
    Vec2::new(TILE_SIZE * 0.25, TILE_SIZE * 0.25),
    Vec2::new(TILE_SIZE * 0.25, -TILE_SIZE * 0.25),
    Vec2::new(-TILE_SIZE * 0.25, -TILE_SIZE * 0.25),
    Vec2::new(-TILE_SIZE * 0.25, TILE_SIZE * 0.25),
];

#[derive(Component)]
struct TallGrass;
#[derive(Component)]
struct NumberPopUp {
    move_speed: f32,
    timer: Timer,
}

#[derive(Event)]
pub struct CutTallGrass {
    entity: Entity,
    pub pos: Vec2,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GrassMaterial {
    #[uniform(0)]
    pub texel_size_and_timestamps: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    pub discrete_sine: Option<Handle<Image>>,
    #[texture(5)]
    #[sampler(6)]
    pub discrete_exp_damp: Option<Handle<Image>>,
}

impl Default for NumberPopUp {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            timer: Timer::from_seconds(0.7, TimerMode::Once),
        }
    }
}

impl NumberPopUp {
    fn new(move_speed: f32, time: f32) -> Self {
        Self {
            move_speed,
            timer: Timer::from_seconds(time, TimerMode::Once),
        }
    }
}

impl Material2d for GrassMaterial {
    fn fragment_shader() -> ShaderRef {
        GRASS_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

fn spawn_tall_grass(
    commands: &mut Commands,
    assets: &GameAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<GrassMaterial>,
    images: &Assets<Image>,
    pos: Vec2,
) {
    let image_handle = assets.grass.clone();
    let Some(image) = images.get(&image_handle) else {
        error!("failed to get grass image from handle, must never happen!");
        return;
    };

    let image_size = Vec2::new(image.width() as f32, image.height() as f32);

    commands.spawn((
        TallGrass,
        YSort(0.0),
        Transform::from_translation(pos.extend(0.0)).with_scale(image_size.extend(1.0)),
        // TODO: Spawn once and set to handle
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(
            materials
                .add(GrassMaterial {
                    texel_size_and_timestamps: (1.0 / image_size).extend(-10.0).extend(-10.0),
                    texture: Some(image_handle.clone()),
                    discrete_sine: Some(assets.discrete_sine_texture.clone()),
                    discrete_exp_damp: Some(assets.discrete_exp_damp_texture.clone()),
                })
                .clone(),
        ),
        StaticSensorAABB::new(8.0, 8.0),
        GRASS_COLLISION_GROUPS,
    ));
}

fn spawn_tall_grass_in_quads(
    commands: &mut Commands,
    assets: &GameAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<GrassMaterial>,
    images: &Assets<Image>,
    center_pos: Vec2,
) {
    let mut rng = thread_rng();
    let mut threshold = 0.35;
    for offset in QUAD_OFFSETS {
        let threshold_check = rng.gen_range(0.0..1.0);

        if threshold_check > threshold {
            threshold += 0.3;
            continue;
        }

        let random_shift = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
        let pos = center_pos + offset + random_shift * QUAD_MAX_SHIFT_OFFSET;
        spawn_tall_grass(commands, assets, meshes, materials, images, pos);
    }
}

fn spawn_grass(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GrassMaterial>>,
    images: Res<Assets<Image>>,
    map_data: Res<MapData>,
) {
    let size = (MAP_SIZE / 2) as i32;
    for i in -size..size {
        for j in -size..size {
            let center_pos = Vec2::new(i as f32 * TILE_SIZE, j as f32 * TILE_SIZE);

            let (x, y) = map_data.pos_to_grid_indices(center_pos);
            if map_data.grid_index(x, y) != TALL_GRASS_CELL_VALUE {
                continue;
            }

            spawn_tall_grass_in_quads(
                &mut commands,
                &assets,
                &mut meshes,
                &mut materials,
                &images,
                center_pos,
            );
        }
    }
}

fn respawn_grass_on_reset(
    mut commands: Commands,
    assets: Res<GameAssets>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<GrassMaterial>>,
    images: Res<Assets<Image>>,
    map_data: Res<MapData>,
    q_grass: Query<Entity, With<TallGrass>>,
    mut ev_menu_action: EventReader<MenuActionEvent>,
) {
    if !ev_menu_action
        .read()
        .any(|ev| ev.action == MenuAction::Reset)
    {
        return;
    }

    for entity in &q_grass {
        commands.entity(entity).despawn();
    }

    spawn_grass(commands, assets, meshes, materials, images, map_data);
}

fn despawn_tall_grass(mut commands: Commands, mut ev_cut_tall_grass: EventReader<CutTallGrass>) {
    for ev in ev_cut_tall_grass.read() {
        commands.entity(ev.entity).despawn();
    }
}

fn trigger_cut_tall_grass_event(
    q_grass: Query<&Transform, With<TallGrass>>,
    mut ev_intersection: EventReader<IntersectionEvent>,
    mut ev_cut_tall_grass: EventWriter<CutTallGrass>,
) {
    for ev in ev_intersection.read() {
        let (entity, other_group) = if ev.collision_groups.0 == GRASS_COLLISION_GROUPS {
            (ev.entities.0, ev.collision_groups.1)
        } else if ev.collision_groups.1 == GRASS_COLLISION_GROUPS {
            (ev.entities.1, ev.collision_groups.0)
        } else {
            continue;
        };

        if other_group != SLASH_COLLISION_GROUPS {
            continue;
        }

        let Ok(transform) = q_grass.get(entity) else {
            continue;
        };
        ev_cut_tall_grass.write(CutTallGrass {
            entity,
            pos: transform.translation.xy(),
        });
    }
}

fn spawn_number_pop_up(
    commands: &mut Commands,
    assets: &GameAssets,
    pos: Vec2,
    text: String,
    color: Color,
    pop_up: NumberPopUp,
    font_size: f32,
    z_index_offset: f32,
) {
    commands.spawn((
        pop_up,
        Text2d(text),
        TextFont {
            font: assets.pixel_font.clone(),
            font_size,
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        TextColor(color),
        Transform::from_translation(pos.extend(ZLevel::TopUi.value() + z_index_offset))
            .with_scale(Vec3::splat(0.1)),
    ));
}

fn spawn_number_pop_ups(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut core: ResMut<ProgressionCore>,
    mut ev_cut_tall_grass: EventReader<CutTallGrass>,
) {
    for ev in ev_cut_tall_grass.read() {
        debug_assert!(core.points <= core.points_cap);
        if core.points == core.points_cap {
            continue;
        }

        core.points += CUT_TALL_GRASS_POINTS;

        spawn_number_pop_up(
            &mut commands,
            &assets,
            ev.pos,
            format!("+{}", CUT_TALL_GRASS_POINTS),
            Color::WHITE.with_alpha(1.0),
            NumberPopUp::default(),
            80.0,
            0.0,
        );
    }
}

/// We spawn the offline progress number pop up in here because it's convenient.
/// It's not clean at all, but I don't care, it's easy to do right now.
fn spawn_offline_progress_number_pop_up(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut core: ResMut<ProgressionCore>,
    q_player: Query<&Transform, With<Player>>,
) {
    if core.offline_progression == 0 {
        return;
    }

    let Ok(player_transform) = q_player.single() else {
        return;
    };

    spawn_number_pop_up(
        &mut commands,
        &assets,
        player_transform.translation.xy() + OFFLINE_PROGRESSION_NUMBER_POP_UP_OFFSET,
        "+".to_string() + &format_money_string_raw(core.offline_progression),
        Color::WHITE.with_alpha(1.0),
        NumberPopUp::default(),
        80.0,
        0.0,
    );
    core.offline_progression = 0;
}

/// We spawn the item bought cost number pop up in here because it's convenient.
/// It's not clean at all, but I don't care, it's easy to do right now.
/// Same reason as for the offline progress.
fn spawn_item_cost_number_pop_up_on_item_bought(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_item_bought: EventReader<ItemBought>,
) {
    for ev in ev_item_bought.read() {
        spawn_number_pop_up(
            &mut commands,
            &assets,
            ev.pos,
            format!("-{}", ev.cost),
            RED.with_alpha(1.0).into(),
            NumberPopUp::new(2.5, 3.0),
            100.0,
            1000.0,
        );
    }
}

fn animate_number_pop_ups(
    time: Res<Time>,
    mut q_pop_ups: Query<(&mut Transform, &mut TextColor, &mut NumberPopUp)>,
) {
    for (mut transform, mut color, mut pop_up) in &mut q_pop_ups {
        pop_up.timer.tick(time.delta());
        transform.translation.y += pop_up.move_speed * time.delta_secs();
        color.set_alpha(1.0 - pop_up.timer.elapsed_secs() / pop_up.timer.duration().as_secs_f32());
    }
}

fn despawn_number_pop_ups(mut commands: Commands, q_pop_ups: Query<(Entity, &NumberPopUp)>) {
    for (entity, pop_up) in &q_pop_ups {
        if pop_up.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn set_grass_timestamps(
    time: Res<Time>,
    mut materials: ResMut<Assets<GrassMaterial>>,
    q_player: Query<(&Transform, &Velocity, &DynamicCollider), With<Player>>,
    q_grass: Query<(&Transform, &MeshMaterial2d<GrassMaterial>), Without<Player>>,
) {
    let Ok((player_transform, player_velocity, player_collider)) = q_player.single() else {
        return;
    };

    // We only want to set the timestamps when the player walks into or away from grass.
    if player_velocity.0 == Vec2::ZERO {
        return;
    }

    let player_pos = player_transform.translation.xy() + player_collider.offset;

    for (grass_transform, material_handle) in &q_grass {
        let grass_pos = grass_transform.translation.xy();

        if player_pos.distance_squared(grass_pos) > (player_collider.radius + 15.0).powi(2) {
            continue;
        }

        let Some(grass_material) = materials.get_mut(material_handle) else {
            continue;
        };

        let current_time = time.elapsed_secs();
        let time_diff = current_time - grass_material.texel_size_and_timestamps.w;
        debug_assert!(time_diff > 0.0);

        // Set exp damp timestamp
        grass_material.texel_size_and_timestamps.w = current_time;

        // Set sine timestamp
        if time_diff > TIME_TILL_SINE_RESET {
            if player_pos.x < grass_pos.x {
                grass_material.texel_size_and_timestamps.z = -current_time;
            } else {
                grass_material.texel_size_and_timestamps.z = current_time;
            }
        }
    }
}

pub struct MapGrassPlugin;

impl Plugin for MapGrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<GrassMaterial>::default())
            .add_event::<CutTallGrass>()
            .add_systems(
                Update,
                (
                    spawn_grass.run_if(resource_exists::<InitialFloraSpawned>.and(run_once)),
                    respawn_grass_on_reset.after(ProgressionSystemSet),
                )
                    .run_if(resource_exists::<GameAssets>.and(resource_exists::<MapData>)),
            )
            .add_systems(
                Update,
                (
                    trigger_cut_tall_grass_event,
                    despawn_tall_grass,
                    spawn_number_pop_ups.run_if(
                        resource_exists::<GameAssets>.and(resource_exists::<ProgressionCore>),
                    ),
                    spawn_offline_progress_number_pop_up.run_if(
                        in_state(GameState::Gaming).and(resource_exists::<ProgressionCore>),
                    ),
                    spawn_item_cost_number_pop_up_on_item_bought.run_if(
                        resource_exists::<GameAssets>.and(resource_exists::<ProgressionCore>),
                    ),
                    animate_number_pop_ups,
                    despawn_number_pop_ups,
                )
                    .chain()
                    .before(ProgressionSystemSet),
            )
            .add_systems(PostUpdate, set_grass_timestamps);
    }
}
