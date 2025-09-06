use std::io::Write;
use std::time::{Duration, Instant};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor, event,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use unicode_width::UnicodeWidthStr;

pub fn tap_tempo_blocking() -> Option<u16> {
    let mut taps: Vec<Instant> = Vec::new();
    let _ = terminal::enable_raw_mode();
    let mut out = std::io::stdout();
    let _ = out.execute(crossterm::terminal::EnterAlternateScreen);
    let _ = out.execute(crossterm::terminal::DisableLineWrap);
    let _ = out.execute(crossterm::cursor::Hide);
    let mut flash_until: Option<Instant> = None;
    loop {
        let (w, h) = terminal::size().unwrap_or((80, 24));
        let mid_y = h / 2;

        let est_bpm = if taps.len() >= 2 {
            let mut intervals = Vec::new();
            for w in taps.windows(2) {
                if let [a, b] = w {
                    intervals.push((*b - *a).as_secs_f64());
                }
            }
            if intervals.is_empty() {
                None
            } else {
                Some(
                    (60.0 / (intervals.iter().sum::<f64>() / intervals.len() as f64)).round()
                        as u16,
                )
            }
        } else {
            None
        };

        let _ = out.queue(Clear(ClearType::All));
        let title = "Tap tempo — <Space> tap, <Enter> accept, <Esc> cancel";
        let info = format!(
            "Taps: {}/8   BPM: {}",
            taps.len().min(8),
            est_bpm.map(|v| v.to_string()).unwrap_or("--".to_string())
        );
        let progress: String = (0..8)
            .map(|i| if i < taps.len().min(8) { '●' } else { '○' })
            .collect();
        let lines = [title.to_string(), info, progress];
        for (i, line) in lines.iter().enumerate() {
            let line_w = UnicodeWidthStr::width(line.as_str()) as u16;
            let x = if w > line_w { (w - line_w) / 2 } else { 0 };
            let y = mid_y + i as u16 - 1;
            let _ = out.queue(cursor::MoveTo(x, y));
            let _ = out.queue(SetForegroundColor(if i == 0 {
                Color::Cyan
            } else {
                Color::Yellow
            }));
            let _ = write!(out, "{}", line);
            let _ = out.queue(ResetColor);
        }

        if let Some(until) = flash_until {
            if Instant::now() < until {
                let _ = out.queue(cursor::MoveTo(w / 2, mid_y + 1));
                let _ = out.queue(SetForegroundColor(Color::Yellow));
                let _ = write!(out, "●");
                let _ = out.queue(ResetColor);
            }
        }
        let _ = out.flush();

        if let Ok(true) = event::poll(Duration::from_millis(16)) {
            if let Ok(ev) = event::read() {
                if let event::Event::Key(key) = ev {
                    use crossterm::event::KeyCode;
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            let _ = out.execute(crossterm::cursor::Show);
                            let _ = out.execute(crossterm::terminal::EnableLineWrap);
                            let _ = out.execute(crossterm::terminal::LeaveAlternateScreen);
                            let _ = terminal::disable_raw_mode();
                            return None;
                        }
                        KeyCode::Enter => {
                            if taps.len() >= 4 {
                                break;
                            }
                        }
                        KeyCode::Char(' ') => {
                            let now = Instant::now();
                            taps.push(now);
                            flash_until = Some(now + Duration::from_millis(150));
                            if taps.len() >= 8 {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    let _ = out.execute(crossterm::cursor::Show);
    let _ = out.execute(crossterm::terminal::EnableLineWrap);
    let _ = out.execute(crossterm::terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
    if taps.len() < 4 {
        return None;
    }
    let mut intervals = Vec::new();
    for w in taps.windows(2) {
        if let [a, b] = w {
            intervals.push((*b - *a).as_secs_f64());
        }
    }
    if intervals.is_empty() {
        return None;
    }
    let avg = intervals.iter().sum::<f64>() / intervals.len() as f64;
    let bpm = (60.0 / avg).round() as u16;
    Some(bpm.clamp(20, 400))
}
