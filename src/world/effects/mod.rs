use bevy::prelude::*;
use bevy_enoki::prelude::*;

use crate::{
    world::{map::CutTallGrass, ProgressionSystemSet, ZLevel},
    EffectAssets,
};

pub struct WorldEffectsPlugin;

impl Plugin for WorldEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_cut_grass_effect.run_if(resource_exists::<EffectAssets>),)
                .after(ProgressionSystemSet),
        );
    }
}

fn spawn_cut_grass_effect(
    mut commands: Commands,
    effects: Res<EffectAssets>,
    mut ev_cut_tall_grass: EventReader<CutTallGrass>,
) {
    for ev in ev_cut_tall_grass.read() {
        commands.spawn((
            Transform::from_translation(ev.pos.extend(ZLevel::TopEnvironment.value())),
            ParticleEffectHandle(effects.cut_grass_particles.clone()),
            ParticleSpawner(effects.cut_grass_material.clone()),
            OneShot::Despawn,
        ));
    }
}
