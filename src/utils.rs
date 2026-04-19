use std::{fs::File, io::BufReader};

use ratatui::layout::{Constraint, Layout, Rect};
use rodio::Decoder;

use crate::state::{App, SortField, SortOrder, SortState};

pub fn get_track_source(idx: usize, app_state: &App) -> Option<Decoder<BufReader<File>>> {
    if let Some(track) = app_state.library.tracks.get(idx)
        && let Ok(file) = File::open(&track.path)
        && let Ok(source) = Decoder::try_from(file)
    {
        return Some(source);
    }

    None
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}

pub fn sort_indicator(label: &str, field: SortField, sort_state: &SortState) -> String {
    if field == sort_state.field {
        let arrow = match sort_state.order {
            SortOrder::Asc => "▲",
            SortOrder::Desc => "▼",
        };

        format!("{label} {arrow}")
    } else {
        label.to_string()
    }
}
