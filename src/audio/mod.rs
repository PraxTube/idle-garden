use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use rand::{thread_rng, Rng};

use crate::{assets::PlayerFootstepEvent, world::ProgressionCore, GameAssets};

const DEFAULT_MUSIC_VOLUME: f32 = 0.05;
const DEFAULT_SOUND_VOLUME: f32 = 0.3;
const MIN_SONG_DURATION: f32 = 180.0;
const MAX_SONG_DURATION: f32 = 300.0;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_bgm.run_if(
                resource_exists::<GameAssets>
                    .and(resource_exists::<ProgressionCore>)
                    .and(run_once),
            ),
        )
        .add_systems(
            Update,
            (
                fade_in,
                fade_out,
                change_to_next_song,
                change_fade_on_core_music,
                spawn_player_footstep_sound,
                silence_all_sounds,
            )
                .chain()
                .run_if(resource_exists::<GameAssets>),
        );
    }
}

#[derive(Component)]
struct Bgm {
    fade_in: bool,
    fade_out: bool,
    fade_time: f32,
    next_song: Timer,
    current_song_index: usize,
    is_active: bool,
}
#[derive(Component)]
struct Sound;

fn spawn_bgm(mut commands: Commands, assets: Res<GameAssets>, core: Res<ProgressionCore>) {
    let mut rng = thread_rng();
    let current_song_index = rng.gen_range(0..assets.bgms.len());

    commands.spawn((
        Bgm {
            fade_in: core.music,
            fade_out: false,
            fade_time: 3.0,
            next_song: Timer::from_seconds(
                rng.gen_range(MIN_SONG_DURATION..MAX_SONG_DURATION),
                TimerMode::Once,
            ),
            current_song_index,
            is_active: core.music,
        },
        AudioPlayer(assets.bgms[current_song_index].clone()),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::SILENT,
            ..default()
        },
    ));
}

fn fade_in(time: Res<Time>, q_bgm: Single<(&mut AudioSink, &mut Bgm)>) {
    let (mut audio, mut bgm) = q_bgm.into_inner();

    if !bgm.fade_in {
        return;
    }
    if !bgm.is_active {
        return;
    }

    debug_assert!(!bgm.fade_out);

    let current_volume = audio.volume().to_linear();
    let new_volume = current_volume + DEFAULT_MUSIC_VOLUME * time.delta_secs() / bgm.fade_time;
    audio.set_volume(Volume::Linear(new_volume));
    if new_volume >= DEFAULT_MUSIC_VOLUME {
        bgm.fade_in = false;
        audio.set_volume(Volume::Linear(DEFAULT_MUSIC_VOLUME));
    }
}

fn fade_out(
    mut commands: Commands,
    assets: Res<GameAssets>,
    time: Res<Time>,
    q_bgm: Single<(Entity, &mut AudioSink, &mut Bgm)>,
) {
    let (entity, mut audio, mut bgm) = q_bgm.into_inner();

    if !bgm.fade_out {
        return;
    }

    debug_assert!(!bgm.fade_in);

    let current_volume = audio.volume().to_linear();
    let new_volume = current_volume - DEFAULT_MUSIC_VOLUME * time.delta_secs() / bgm.fade_time;
    audio.set_volume(Volume::Linear(new_volume));
    if new_volume <= 0.0 {
        bgm.fade_out = false;
        bgm.fade_in = true;
        bgm.fade_time = 10.0;
        audio.set_volume(Volume::SILENT);

        let mut rng = thread_rng();
        let shifted_index = rng.gen_range(0..assets.bgms.len() - 1);

        let index = if shifted_index >= bgm.current_song_index {
            shifted_index + 1
        } else {
            shifted_index
        };

        debug_assert_ne!(bgm.current_song_index, index);

        bgm.current_song_index = index;
        commands
            .entity(entity)
            .remove::<AudioSink>()
            .insert(AudioPlayer(assets.bgms[index].clone()));
    }
}

fn change_to_next_song(time: Res<Time>, q_bgm: Single<&mut Bgm>) {
    let mut bgm = q_bgm.into_inner();
    if !bgm.is_active {
        return;
    }

    bgm.next_song.tick(time.delta());
    if bgm.next_song.just_finished() {
        let mut rng = thread_rng();
        bgm.next_song = Timer::from_seconds(
            rng.gen_range(MIN_SONG_DURATION..MAX_SONG_DURATION),
            TimerMode::Once,
        );
        bgm.fade_time = 10.0;
        bgm.fade_out = true;
    }
}

fn change_fade_on_core_music(core: Res<ProgressionCore>, q_bgm: Single<&mut Bgm>) {
    let mut bgm = q_bgm.into_inner();
    if core.music != bgm.is_active {
        bgm.is_active = core.music;
        if core.music {
            bgm.fade_time = 10.0;
            bgm.fade_in = true;
            bgm.fade_out = false;
        } else {
            bgm.fade_time = 0.5;
            bgm.fade_out = true;
            bgm.fade_in = false;
        }
    }
}

fn spawn_player_footstep_sound(
    mut commands: Commands,
    assets: Res<GameAssets>,
    core: Res<ProgressionCore>,
    mut ev_player_footstep: EventReader<PlayerFootstepEvent>,
) {
    if ev_player_footstep.is_empty() {
        return;
    }
    ev_player_footstep.clear();

    if !core.sound {
        return;
    }

    let mut rng = thread_rng();

    commands.spawn((
        Sound,
        AudioPlayer(assets.player_footstep.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            speed: rng.gen_range(0.7..1.3),
            volume: Volume::Linear(DEFAULT_SOUND_VOLUME * rng.gen_range(0.8..1.0)),
            ..default()
        },
    ));
}

fn silence_all_sounds(
    core: Res<ProgressionCore>,
    mut q_sounds: Query<&mut AudioSink, With<Sound>>,
) {
    if core.sound {
        return;
    }

    for mut audio in &mut q_sounds {
        audio.set_volume(Volume::SILENT);
    }
}
