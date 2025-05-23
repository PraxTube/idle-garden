use bevy::prelude::*;

use crate::{
    player::Player,
    ui::ItemPressed,
    world::{
        camera::YSort,
        collisions::{StaticCollider, WORLD_COLLISION_GROUPS},
        TILE_SIZE,
    },
    GameAssets,
};

use super::MapGrid;

#[derive(Clone, Copy)]
pub enum Flora {
    Potatoe,
    Tree,
    Weed,
}

impl Flora {
    fn grid_value(&self) -> u16 {
        *self as u16
    }

    fn gfx_data(&self, assets: &GameAssets) -> (Handle<Image>, YSort, Vec2) {
        match self {
            Flora::Potatoe => todo!(),
            Flora::Tree => (
                assets.tree.clone(),
                YSort(0.0),
                Vec2::new(0.0, 2.5 * TILE_SIZE),
            ),
            Flora::Weed => todo!(),
        }
    }

    fn collider_data(&self) -> StaticCollider {
        match self {
            Flora::Potatoe => todo!(),
            Flora::Tree => StaticCollider::new(8.0, 6.0),
            Flora::Weed => todo!(),
        }
    }

    fn size_on_grid(&self) -> (usize, usize) {
        match self {
            Flora::Potatoe => todo!(),
            Flora::Tree => (2, 2),
            Flora::Weed => todo!(),
        }
    }

    fn size_offset(&self) -> Vec2 {
        let (x, y) = self.size_on_grid();

        debug_assert!(x > 0);
        debug_assert!(y > 0);

        0.5 * TILE_SIZE * Vec2::new((x - 1) as f32, (y - 1) as f32)
    }
}

fn spawn_flora(commands: &mut Commands, assets: &GameAssets, pos: Vec2, flora: Flora) {
    let (image, ysort, offset) = flora.gfx_data(assets);
    let collider = flora.collider_data();

    let root = commands
        .spawn((
            Transform::from_translation(pos.extend(0.0)),
            ysort,
            Visibility::Inherited,
            collider,
            WORLD_COLLISION_GROUPS,
        ))
        .id();

    commands.spawn((
        ChildOf(root),
        Transform::from_translation(offset.extend(0.0)),
        Sprite::from_image(image),
    ));
}

fn spawn_flora_from_item_pressed(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut map_grid: ResMut<MapGrid>,
    q_player: Query<&Transform, With<Player>>,
    mut ev_item_pressed: EventReader<ItemPressed>,
) {
    let Ok(transform) = q_player.single() else {
        debug_assert!(ev_item_pressed.is_empty());
        return;
    };

    for ev in ev_item_pressed.read() {
        let pos = map_grid.random_grid_position_near_player_pos(
            transform.translation.xy(),
            ev.flora.size_on_grid(),
        );

        map_grid.set_map_grid_value_at_pos(pos, ev.flora.size_on_grid(), ev.flora.grid_value());
        spawn_flora(
            &mut commands,
            &assets,
            pos + ev.flora.size_offset(),
            ev.flora,
        );
    }
}

pub struct MapFloraPlugin;

impl Plugin for MapFloraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_flora_from_item_pressed.run_if(resource_exists::<GameAssets>),
        );
    }
}
