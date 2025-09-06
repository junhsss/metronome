use crossterm::{
    QueueableCommand, cursor,
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::Write;
use unicode_width::UnicodeWidthStr;

pub fn render_ui(
    out: &mut std::io::Stdout,
    width: u16,
    height: u16,
    bpm: u16,
    bar_beats: u8,
    denom: u8,
    ticks_per_beat: u8,
    beat_in_bar: u8,
    tick_in_beat: u8,
    playing: bool,
    accent: bool,
    show_help: bool,
) {
    fn render_tokens(out: &mut std::io::Stdout, text: &str) {
        let mut in_token = false;
        for ch in text.chars() {
            match ch {
                '<' => {
                    let _ = out.queue(SetAttribute(Attribute::Bold));
                    let _ = write!(out, "<");
                    in_token = true;
                }
                '>' if in_token => {
                    let _ = write!(out, ">");
                    let _ = out.queue(SetAttribute(Attribute::Reset));
                    in_token = false;
                }
                _ => {
                    let _ = write!(out, "{}", ch);
                }
            }
        }
        if in_token {
            let _ = out.queue(SetAttribute(Attribute::Reset));
        }
    }
    let total_ticks = (bar_beats as u32) * (ticks_per_beat as u32);
    let current_tick_index =
        ((beat_in_bar as u32 - 1) * ticks_per_beat as u32) + tick_in_beat as u32;

    let title = "Metronome";
    let hud_state = if playing { "RUN" } else { "PAUSE" };
    let hud_text = format!(
        "{:>3} BPM  |  {}/{}  |  sub {}  |  {}",
        bpm, bar_beats, denom, ticks_per_beat, hud_state
    );

    let _ = out.queue(Clear(ClearType::All));
    let _ = out.queue(cursor::MoveTo(0, 0));
    let _ = out.queue(SetAttribute(Attribute::Bold));
    let _ = out.queue(SetForegroundColor(Color::Cyan));
    let _ = write!(out, "{}", title);
    let _ = out.queue(ResetColor);
    let _ = out.queue(SetAttribute(Attribute::Reset));
    let right_text = hud_text;
    if width > 0 {
        let right_col = width.saturating_sub(UnicodeWidthStr::width(right_text.as_str()) as u16);
        let _ = out.queue(cursor::MoveTo(right_col, 0));
    }
    let _ = out.queue(SetAttribute(Attribute::Bold));
    let _ = out.queue(SetForegroundColor(Color::Yellow));
    let _ = write!(out, "{}", right_text);
    let _ = out.queue(ResetColor);
    let _ = out.queue(SetAttribute(Attribute::Reset));

    let bar_top = 2u16;
    let bar_bottom = height.saturating_sub(3).max(bar_top);
    for row in bar_top..=bar_bottom {
        let _ = out.queue(cursor::MoveTo(0, row));
        let mut bar: Vec<char> = vec![' '; width as usize];
        if width > 0 && total_ticks > 0 {
            for b in 0..=bar_beats as u32 {
                let pos = ((b * ticks_per_beat as u32) * width as u32) / total_ticks;
                let idx = pos.min(width as u32 - 1) as usize;
                bar[idx] = '|';
            }
            for b in 0..bar_beats as u32 {
                let left = ((b * ticks_per_beat as u32) * width as u32) / total_ticks;
                let right = (((b + 1) * ticks_per_beat as u32) * width as u32) / total_ticks;
                let left_i = left.min(width as u32 - 1) as usize;
                let right_i = right.min(width as u32 - 1) as usize;
                let fill_char = if (b + 1) == beat_in_bar as u32 {
                    '='
                } else {
                    '-'
                };
                for i in left_i.saturating_add(1)..right_i {
                    bar[i] = fill_char;
                }
            }
            let tick_pos = (current_tick_index * width as u32) / total_ticks;
            let tick_idx = tick_pos.min(width as u32 - 1) as usize;
            bar[tick_idx] = '●';
        }
        let line: String = bar.into_iter().collect();
        let _ = out.queue(SetForegroundColor(Color::DarkBlue));
        let _ = write!(out, "{}", line);
        let _ = out.queue(cursor::MoveTo(0, row));
        let _ = out.queue(SetForegroundColor(Color::Blue));
        for b in 0..=bar_beats as u32 {
            let pos = ((b * ticks_per_beat as u32) * width as u32) / total_ticks;
            let idx = pos.min(width as u32 - 1) as u16;
            let _ = out.queue(cursor::MoveTo(idx, row));
            let _ = write!(out, "|");
        }
        let tick_pos = (current_tick_index * width as u32) / total_ticks;
        let tick_x = tick_pos.min(width as u32 - 1) as u16;
        let _ = out.queue(cursor::MoveTo(tick_x, row));
        let _ = out.queue(SetForegroundColor(
            if accent && tick_in_beat == 0 && beat_in_bar == 1 {
                Color::Yellow
            } else {
                Color::Cyan
            },
        ));
        let _ = write!(out, "●");
        let _ = out.queue(ResetColor);
    }

    let left_help =
        "<Space>: Play/Pause   <q>/<Esc>: Quit   <s>: Subdivision   <Tab>: Signature   <h>: Help";
    let right_help = "<↑>/<↓>: ±1   <←>/<→>: ±5";
    let help_y = height.saturating_sub(1);
    let _ = out.queue(cursor::MoveTo(0, help_y));
    let _ = out.queue(SetForegroundColor(Color::DarkGrey));
    let _ = out.queue(Clear(ClearType::CurrentLine));

    let left_render_width = UnicodeWidthStr::width(left_help) as u16;
    if left_render_width >= width {
        let max = width.saturating_sub(1) as usize;
        let truncated = &left_help[..left_help
            .chars()
            .take(max)
            .map(|c| c.len_utf8())
            .sum::<usize>()
            .min(left_help.len())];
        render_tokens(out, truncated);
    } else {
        render_tokens(out, left_help);
    }

    let right_w = UnicodeWidthStr::width(right_help) as u16;
    if width > right_w + 2 {
        let right_start = width - right_w;
        let _ = out.queue(cursor::MoveTo(right_start, help_y));
        if right_w >= width {
            let max = width.saturating_sub(1) as usize;
            let truncated = &right_help[..right_help
                .chars()
                .take(max)
                .map(|c| c.len_utf8())
                .sum::<usize>()
                .min(right_help.len())];
            render_tokens(out, truncated);
        } else {
            render_tokens(out, right_help);
        }
    }

    if show_help && height > 4 {
        let box_top = height.saturating_sub(5);
        let _ = out.queue(cursor::MoveTo(0, box_top));
        let _ = out.queue(SetForegroundColor(Color::DarkGrey));
        let _ = out.queue(Clear(ClearType::FromCursorDown));
        let lines = [
            "Help:",
            "  <Space> Play/Pause   <q>/<Esc> Quit",
            "  <s> Subdivision   <Tab> Signature",
            "  <↑>/<↓> ±1   <←>/<→> ±5",
        ];
        for (i, l) in lines.iter().enumerate() {
            let _ = out.queue(cursor::MoveTo(0, box_top + i as u16));
            render_tokens(out, l);
        }
        let _ = out.queue(ResetColor);
    }
    let _ = out.queue(ResetColor);
    let _ = out.flush();
}
