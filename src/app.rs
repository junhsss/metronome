use std::io::stdout;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::terminal;
use crossterm::{ExecutableCommand, event};

use crate::audio::spawn_audio_thread;
use crate::cli::{Cli, Commands, SoundType, Subdivision};
use crate::tap::tap_tempo_blocking;
use crate::tempo::{parse_duration_ms, parse_ramp_pattern, parse_signature};
use crate::ui::render_ui;

pub fn run(mut cli: Cli) {
    if let Some(pos) = cli.bpm_positional {
        cli.bpm = pos;
    }
    if let Some(Commands::Tap { apply: _ }) = &cli.command {
        if let Some(bpm) = tap_tempo_blocking() {
            cli.bpm = bpm;
        }
    }

    let (mut numerator, mut denominator) = match parse_signature(&cli.signature) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(2);
        }
    };

    let mut ticks_per_beat = cli.subdivision.ticks_per_beat();
    println!(
        "Starting metronome: {} BPM | {}/{} | subdivision: {} per beat | mute: {}",
        cli.bpm, numerator, denominator, ticks_per_beat, cli.mute
    );

    let mut beat_in_bar: u8 = 1;
    let mut tick_in_beat: u8 = 0;

    let (audio_tx, audio_rx) = mpsc::channel::<(bool, SoundType)>();
    if !cli.mute {
        spawn_audio_thread(audio_rx);
    }

    let mut next_tick = Instant::now();
    let mut playing = true;
    let mut show_help = false;

    let mut ramp_from_bpm: Option<(u16, u16, u64, Instant)> = None;
    if let Some(Commands::Ramp { pattern }) = &cli.command {
        if let Some(cfg) = parse_ramp_pattern(pattern, parse_duration_ms) {
            ramp_from_bpm = Some((cfg.from_bpm, cfg.to_bpm, cfg.duration_ms, Instant::now()));
            cli.bpm = cfg.from_bpm;
        }
    }

    let _ = terminal::enable_raw_mode();
    let term_restored = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    {
        let term_restored = term_restored.clone();
        let _ = ctrlc::set_handler(move || {
            if !term_restored.swap(true, std::sync::atomic::Ordering::SeqCst) {
                cleanup_terminal();
            }
            std::process::exit(0);
        });
    }
    std::panic::set_hook({
        let term_restored = term_restored.clone();
        Box::new(move |info| {
            if !term_restored.swap(true, std::sync::atomic::Ordering::SeqCst) {
                cleanup_terminal();
            }
            eprintln!("panic: {}", info);
        })
    });
    let mut stdout_handle = stdout();
    let _ = stdout_handle.execute(crossterm::terminal::EnterAlternateScreen);
    let _ = stdout_handle.execute(crossterm::terminal::DisableLineWrap);
    let _ = stdout_handle.execute(crossterm::cursor::Hide);

    loop {
        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(ev) = event::read() {
                if let event::Event::Key(key) = ev {
                    use crossterm::event::KeyCode;
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            cleanup_terminal();
                            return;
                        }
                        KeyCode::Char(' ') => {
                            playing = !playing;
                        }
                        KeyCode::Up => {
                            if cli.bpm < 400 {
                                cli.bpm = (cli.bpm + 1).min(400);
                            }
                        }
                        KeyCode::Down => {
                            if cli.bpm > 20 {
                                cli.bpm = (cli.bpm.saturating_sub(1)).max(20);
                            }
                        }
                        KeyCode::Right => {
                            cli.bpm = (cli.bpm + 5).min(400);
                        }
                        KeyCode::Left => {
                            cli.bpm = cli.bpm.saturating_sub(5).max(20);
                        }
                        KeyCode::Char('s') => {
                            cli.subdivision = match cli.subdivision {
                                Subdivision::Quarter => Subdivision::Eighth,
                                Subdivision::Eighth => Subdivision::Triplet,
                                Subdivision::Triplet => Subdivision::Sixteenth,
                                Subdivision::Sixteenth => Subdivision::Quarter,
                            };
                            ticks_per_beat = cli.subdivision.ticks_per_beat();
                        }
                        KeyCode::Tab => {
                            let next = match (numerator, denominator) {
                                (4, 4) => (3, 4),
                                (3, 4) => (6, 8),
                                (6, 8) => (7, 8),
                                _ => (4, 4),
                            };
                            numerator = next.0;
                            denominator = next.1;
                            if beat_in_bar > numerator {
                                beat_in_bar = 1;
                            }
                        }
                        KeyCode::Char('h') => {
                            show_help = !show_help;
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Some((from, to, dur_ms, start)) = ramp_from_bpm {
            let elapsed = Instant::now().saturating_duration_since(start).as_millis() as u64;
            let ratio = (elapsed as f64 / dur_ms.max(1) as f64).min(1.0);
            let bpm_now = from as f64 + (to as f64 - from as f64) * ratio;
            cli.bpm = bpm_now.round().clamp(20.0, 400.0) as u16;
        }

        let beats_per_second = cli.bpm as f64 / 60.0;
        let ticks_per_second = beats_per_second * ticks_per_beat as f64;
        let nanos_per_tick = (1_000_000_000f64 / ticks_per_second) as u64;
        let base_tick_duration = Duration::from_nanos(nanos_per_tick);

        let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
        if playing {
            let is_accent = tick_in_beat == 0 && beat_in_bar == 1;
            render_ui(
                &mut stdout_handle,
                term_w,
                term_h,
                cli.bpm,
                numerator,
                denominator,
                ticks_per_beat,
                beat_in_bar,
                tick_in_beat,
                true,
                is_accent,
                show_help,
            );
            if !cli.mute {
                let _ = audio_tx.send((is_accent, cli.sound));
            }
            let tick_duration = base_tick_duration;
            next_tick += tick_duration;
        } else {
            render_ui(
                &mut stdout_handle,
                term_w,
                term_h,
                cli.bpm,
                numerator,
                denominator,
                ticks_per_beat,
                beat_in_bar,
                tick_in_beat,
                false,
                false,
                show_help,
            );
            next_tick = Instant::now() + base_tick_duration;
        }

        let now = Instant::now();
        if next_tick > now {
            let mut remaining = next_tick - now;
            while remaining > Duration::from_millis(2) {
                thread::sleep(remaining - Duration::from_millis(2));
                let now_inner = Instant::now();
                if next_tick <= now_inner {
                    break;
                }
                remaining = next_tick - now_inner;
            }
            while Instant::now() < next_tick {}
        } else if playing {
            let behind = now - next_tick;
            let tick_duration = base_tick_duration;
            let ticks_behind = (behind.as_nanos() / tick_duration.as_nanos().max(1)) as u64;
            next_tick = now + tick_duration;
            if ticks_behind > 0 {
                tick_in_beat =
                    tick_in_beat.saturating_add((ticks_behind % ticks_per_beat as u64) as u8);
                while tick_in_beat >= ticks_per_beat {
                    tick_in_beat -= ticks_per_beat;
                    beat_in_bar += 1;
                    if beat_in_bar > numerator {
                        beat_in_bar = 1;
                    }
                }
            }
        } else {
            next_tick = now + base_tick_duration;
        }

        if playing {
            tick_in_beat += 1;
            if tick_in_beat >= ticks_per_beat {
                tick_in_beat = 0;
                beat_in_bar += 1;
                if beat_in_bar > numerator {
                    beat_in_bar = 1;
                }
            }
        }
    }
}

pub fn cleanup_terminal() {
    let mut stdout_handle = stdout();
    let _ = stdout_handle.execute(crossterm::cursor::Show);
    let _ = stdout_handle.execute(crossterm::terminal::EnableLineWrap);
    let _ = stdout_handle.execute(crossterm::terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}
