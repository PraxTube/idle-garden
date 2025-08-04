use std::fs::{self};

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_enoki::prelude::*;
use crossbeam::channel::{bounded, Receiver, Sender};
use ron::ser::PrettyConfig;
use std::time::Duration;

pub struct FileManagerPlugin;
impl Plugin for FileManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EffectChannel>();
        app.add_systems(
            Update,
            (effect_file_watcher,).run_if(on_timer(Duration::from_millis(50))),
        );
    }
}

#[derive(Resource)]
pub(crate) struct EffectChannel {
    pub last_file_name: String,
    pub send: Sender<EffectFileWrapper>,
    rec: Receiver<EffectFileWrapper>,
}

pub struct EffectFileWrapper {
    file_name: String,
    effect: Particle2dEffect,
}

impl Default for EffectChannel {
    fn default() -> Self {
        let (tx, rx) = bounded(1);
        Self {
            last_file_name: "my_new_effect.ron".to_string(),
            send: tx,
            rec: rx,
        }
    }
}

fn effect_file_watcher(
    mut effect_channel: ResMut<EffectChannel>,
    mut instances: Query<&mut ParticleEffectInstance>,
) {
    let Ok(effect_wrapper) = effect_channel.rec.try_recv() else {
        return;
    };

    effect_channel.last_file_name = effect_wrapper.file_name;
    instances.iter_mut().for_each(|mut instance| {
        instance.0 = Some(effect_wrapper.effect.clone());
    });
}

pub fn open_effect_file(sender: Sender<EffectFileWrapper>, file: &str) {
    if !fs::exists(file).expect("failed to even verify file's existence, something went very wrong")
    {
        save_effect(Particle2dEffect::default(), file.to_string());
    }

    let content = fs::read(file).expect("failed to read effect file");

    let effect: Particle2dEffect = match ron::de::from_bytes(&content) {
        Ok(effect) => effect,
        Err(err) => {
            error!(
                "`{}` is not a valid particle effect asset!\n\n {:?}",
                file, err
            );
            return;
        }
    };

    let packed_effect = EffectFileWrapper {
        effect,
        file_name: file.to_string(),
    };

    match sender.send(packed_effect) {
        Ok(_) => (),
        Err(err) => {
            error!("Channel failed!\n\n {:?}", err);
        }
    };
}

pub fn save_effect(effect: Particle2dEffect, file_name: String) {
    let content = match ron::ser::to_string_pretty(&effect, PrettyConfig::default()) {
        Ok(b) => b,
        Err(err) => {
            error!(
                "Ops, cannot convert to string, this should not happen!\n\n {:?}",
                err
            );
            return;
        }
    };

    fs::write(&file_name, content).expect("failed to write effect file");
}
