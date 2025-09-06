use std::thread;
use std::time::Duration;

use crate::cli::SoundType;

pub fn spawn_audio_thread(rx: std::sync::mpsc::Receiver<(bool, SoundType)>) {
    thread::spawn(move || {
        use rodio::{OutputStream, Sink, Source, source::SineWave};

        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok(s) => s,
            Err(_) => return,
        };

        let click_ms_accent: u64 = 30;
        let click_ms_weak: u64 = 20;
        let freq_accent: u32 = 1760;
        let freq_weak: u32 = 1320;

        let sink = match Sink::try_new(&stream_handle) {
            Ok(s) => s,
            Err(_) => return,
        };
        sink.pause();
        while let Ok((is_accent, sound)) = rx.recv() {
            let duration_ms = if is_accent {
                click_ms_accent
            } else {
                click_ms_weak
            };
            let freq = match sound {
                SoundType::Click => {
                    if is_accent {
                        freq_accent
                    } else {
                        freq_weak
                    }
                }
                SoundType::Beep => {
                    if is_accent {
                        1760
                    } else {
                        880
                    }
                }
                SoundType::Wood => {
                    if is_accent {
                        1500
                    } else {
                        900
                    }
                }
                SoundType::Cowbell => {
                    if is_accent {
                        2000
                    } else {
                        1200
                    }
                }
                SoundType::Sidestick => {
                    if is_accent {
                        1200
                    } else {
                        800
                    }
                }
            };
            let sine = SineWave::new(freq as f32)
                .take_duration(Duration::from_millis(duration_ms))
                .amplify(0.2);
            sink.append(sine);
            sink.play();
        }
    });
}
