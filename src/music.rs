use rand::seq::SliceRandom;
use rodio::source::{PinkNoise, SineWave};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const MUSIC_VOLUME: f32 = 0.35;
const FOOTSTEP_VOLUME: f32 = 0.16;
const SUCCESS_VOLUME: f32 = 0.14;
const UPDATE_INTERVAL: Duration = Duration::from_millis(33);

#[derive(Clone, Copy)]
struct Track {
    filename: &'static str,
    display_name: &'static str,
}

const TRACKS: [Track; 7] = [
    Track {
        filename: "style.mp3",
        display_name: "STYLE",
    },
    Track {
        filename: "out_of_the_woods.mp3",
        display_name: "OUT OF THE WOODS",
    },
    Track {
        filename: "wildest_dreams.mp3",
        display_name: "WILDEST DREAMS",
    },
    Track {
        filename: "cardigan.mp3",
        display_name: "CARDIGAN",
    },
    Track {
        filename: "all_too_well.mp3",
        display_name: "ALL TOO WELL",
    },
    Track {
        filename: "delicate.mp3",
        display_name: "DELICATE",
    },
    Track {
        filename: "this_love.mp3",
        display_name: "THIS LOVE",
    },
];

#[derive(Debug, Clone, Copy)]
pub struct MusicSnapshot {
    pub track_name: &'static str,
    pub elapsed: Duration,
    pub duration: Duration,
    pub paused: bool,
    pub muted: bool,
    pub available: bool,
}

impl Default for MusicSnapshot {
    fn default() -> Self {
        Self {
            track_name: "CARGANDO MUSICA",
            elapsed: Duration::ZERO,
            duration: Duration::ZERO,
            paused: false,
            muted: false,
            available: false,
        }
    }
}

enum MusicCommand {
    SetGameActive(bool),
    TogglePause,
    Next,
    ToggleMute,
    PlayFootstep,
    PlaySuccess,
    Shutdown,
}

pub struct MusicPlayer {
    commands: mpsc::Sender<MusicCommand>,
    state: Arc<Mutex<MusicSnapshot>>,
    worker: Option<JoinHandle<()>>,
    game_active: bool,
}

impl MusicPlayer {
    pub fn new() -> Self {
        let mut playlist = TRACKS.to_vec();
        playlist.shuffle(&mut rand::thread_rng());

        let (commands, receiver) = mpsc::channel();
        let state = Arc::new(Mutex::new(MusicSnapshot::default()));
        let worker_state = Arc::clone(&state);
        let worker = thread::Builder::new()
            .name("music-playlist".to_owned())
            .spawn(move || run_playlist(playlist, receiver, worker_state))
            .ok();
        if worker.is_none() {
            set_unavailable(&state);
        }

        Self {
            commands,
            state,
            worker,
            game_active: false,
        }
    }

    pub fn sync_game_state(&mut self, active: bool) {
        if active != self.game_active {
            self.game_active = active;
            let _ = self.commands.send(MusicCommand::SetGameActive(active));
        }
    }

    pub fn toggle_pause(&self) {
        let _ = self.commands.send(MusicCommand::TogglePause);
    }

    pub fn next(&self) {
        let _ = self.commands.send(MusicCommand::Next);
    }

    pub fn toggle_mute(&self) {
        let _ = self.commands.send(MusicCommand::ToggleMute);
    }

    pub fn play_footstep(&self) {
        let _ = self.commands.send(MusicCommand::PlayFootstep);
    }

    pub fn play_success(&self) {
        let _ = self.commands.send(MusicCommand::PlaySuccess);
    }

    pub fn snapshot(&self) -> MusicSnapshot {
        self.state
            .lock()
            .map_or_else(|poisoned| *poisoned.into_inner(), |state| *state)
    }
}

impl Drop for MusicPlayer {
    fn drop(&mut self) {
        let _ = self.commands.send(MusicCommand::Shutdown);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn run_playlist(
    playlist: Vec<Track>,
    commands: mpsc::Receiver<MusicCommand>,
    state: Arc<Mutex<MusicSnapshot>>,
) {
    let Ok((_stream, stream_handle)) = OutputStream::try_default() else {
        set_unavailable(&state);
        wait_for_shutdown(commands);
        return;
    };

    let mut current_index = playlist.len().saturating_sub(1);
    let mut game_active = false;
    let mut user_paused = false;
    let mut muted = false;
    let mut alternate_footstep = false;
    let mut playback = load_next_track(
        &stream_handle,
        &playlist,
        &mut current_index,
        game_active,
        user_paused,
        muted,
    );

    loop {
        match commands.recv_timeout(UPDATE_INTERVAL) {
            Ok(MusicCommand::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Ok(command) => {
                if process_command(
                    command,
                    &stream_handle,
                    &playlist,
                    &mut current_index,
                    &mut playback,
                    &mut game_active,
                    &mut user_paused,
                    &mut muted,
                    &mut alternate_footstep,
                ) {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }

        if playback
            .as_ref()
            .is_some_and(|current| current.sink.empty())
        {
            playback = load_next_track(
                &stream_handle,
                &playlist,
                &mut current_index,
                game_active,
                user_paused,
                muted,
            );
        }

        publish_snapshot(&state, playback.as_ref(), user_paused, muted);
    }
}

#[allow(clippy::too_many_arguments)]
fn process_command(
    command: MusicCommand,
    stream_handle: &OutputStreamHandle,
    playlist: &[Track],
    current_index: &mut usize,
    playback: &mut Option<Playback>,
    game_active: &mut bool,
    user_paused: &mut bool,
    muted: &mut bool,
    alternate_footstep: &mut bool,
) -> bool {
    match command {
        MusicCommand::SetGameActive(active) => {
            *game_active = active;
            apply_play_state(playback.as_ref(), *game_active, *user_paused);
        }
        MusicCommand::TogglePause => {
            *user_paused = !*user_paused;
            apply_play_state(playback.as_ref(), *game_active, *user_paused);
        }
        MusicCommand::Next => {
            if let Some(current) = playback.take() {
                current.sink.stop();
            }
            *playback = load_next_track(
                stream_handle,
                playlist,
                current_index,
                *game_active,
                *user_paused,
                *muted,
            );
        }
        MusicCommand::ToggleMute => {
            *muted = !*muted;
            if let Some(current) = playback.as_ref() {
                current
                    .sink
                    .set_volume(if *muted { 0.0 } else { MUSIC_VOLUME });
            }
        }
        MusicCommand::PlayFootstep => {
            play_footstep_effect(stream_handle, *alternate_footstep);
            *alternate_footstep = !*alternate_footstep;
        }
        MusicCommand::PlaySuccess => play_success_effect(stream_handle),
        MusicCommand::Shutdown => return true,
    }
    false
}

fn play_footstep_effect(stream_handle: &OutputStreamHandle, alternate: bool) {
    let Ok(sink) = Sink::try_new(stream_handle) else {
        return;
    };
    let frequency = if alternate { 82.0 } else { 96.0 };
    let duration = Duration::from_millis(105);
    let thump = SineWave::new(frequency)
        .take_duration(duration)
        .fade_out(Duration::from_millis(90))
        .amplify(FOOTSTEP_VOLUME);
    let grit = PinkNoise::new(rodio::cpal::SampleRate(48_000))
        .take_duration(Duration::from_millis(65))
        .fade_out(Duration::from_millis(60))
        .amplify(0.025);

    sink.append(thump.mix(grit));
    sink.detach();
}

fn play_success_effect(stream_handle: &OutputStreamHandle) {
    let Ok(sink) = Sink::try_new(stream_handle) else {
        return;
    };

    for (frequency, milliseconds) in [(523.25, 130), (659.25, 130), (783.99, 320)] {
        sink.append(
            SineWave::new(frequency)
                .take_duration(Duration::from_millis(milliseconds))
                .fade_in(Duration::from_millis(12))
                .fade_out(Duration::from_millis(80))
                .amplify(SUCCESS_VOLUME),
        );
    }
    sink.detach();
}

struct Playback {
    sink: Sink,
    track: Track,
    duration: Duration,
}

fn load_next_track(
    stream_handle: &OutputStreamHandle,
    playlist: &[Track],
    current_index: &mut usize,
    game_active: bool,
    user_paused: bool,
    muted: bool,
) -> Option<Playback> {
    for _ in 0..playlist.len() {
        *current_index = (*current_index + 1) % playlist.len();
        let track = playlist[*current_index];
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/audio")
            .join(track.filename);
        let Ok(file) = File::open(path) else { continue };
        let Ok(decoder) = Decoder::new(BufReader::new(file)) else {
            continue;
        };
        let duration = decoder.total_duration().unwrap_or(Duration::ZERO);
        let Ok(sink) = Sink::try_new(stream_handle) else {
            return None;
        };

        sink.set_volume(if muted { 0.0 } else { MUSIC_VOLUME });
        sink.append(decoder);
        if game_active && !user_paused {
            sink.play();
        } else {
            sink.pause();
        }

        return Some(Playback {
            sink,
            track,
            duration,
        });
    }
    None
}

fn apply_play_state(playback: Option<&Playback>, game_active: bool, user_paused: bool) {
    if let Some(current) = playback {
        if game_active && !user_paused {
            current.sink.play();
        } else {
            current.sink.pause();
        }
    }
}

fn publish_snapshot(
    state: &Mutex<MusicSnapshot>,
    playback: Option<&Playback>,
    user_paused: bool,
    muted: bool,
) {
    let mut snapshot = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(current) = playback {
        *snapshot = MusicSnapshot {
            track_name: current.track.display_name,
            elapsed: current.sink.get_pos().min(current.duration),
            duration: current.duration,
            paused: user_paused,
            muted,
            available: true,
        };
    } else {
        *snapshot = MusicSnapshot {
            track_name: "AUDIO NO DISPONIBLE",
            elapsed: Duration::ZERO,
            duration: Duration::ZERO,
            paused: user_paused,
            muted,
            available: false,
        };
    }
}

fn set_unavailable(state: &Mutex<MusicSnapshot>) {
    let mut snapshot = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    snapshot.track_name = "AUDIO NO DISPONIBLE";
    snapshot.available = false;
}

fn wait_for_shutdown(commands: mpsc::Receiver<MusicCommand>) {
    while !matches!(commands.recv(), Ok(MusicCommand::Shutdown) | Err(_)) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_playlist_declara_las_siete_canciones() {
        assert_eq!(
            TRACKS.map(|track| track.filename),
            [
                "style.mp3",
                "out_of_the_woods.mp3",
                "wildest_dreams.mp3",
                "cardigan.mp3",
                "all_too_well.mp3",
                "delicate.mp3",
                "this_love.mp3",
            ]
        );
    }
}
