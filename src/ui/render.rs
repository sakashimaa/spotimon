use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Row, Table, TableState},
};

use crate::track_library;

fn render_track_table(
    frame: &mut Frame,
    area: Rect,
    table_state: &mut TableState,
    track_library: &track_library::TrackLibrary,
) {
    let header = Row::new(["Title", "Artist", "Album", "Duration"])
        .style(Style::new().bold())
        .bottom_margin(1);

    let rows: Vec<Row> = track_library
        .tracks
        .iter()
        .map(|t| {
            let mins = t.duration.as_secs() / 60;
            let secs = t.duration.as_secs() % 60;
            Row::new([
                t.title.clone(),
                t.artist.clone(),
                t.album.clone(),
                format!("{}:{:02}", mins, secs),
            ])
        })
        .collect();
    let widths = [
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(1)
        .style(Color::Blue)
        .row_highlight_style(Style::new().on_black().bold())
        .column_highlight_style(Color::Gray);

    frame.render_stateful_widget(table, area, table_state);
}

pub fn render(
    frame: &mut Frame,
    library: &track_library::TrackLibrary,
    track_table_state: &mut TableState,
) {
    let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1);
    let [top, main] = frame.area().layout(&layout);

    let title = Line::from_iter([
        Span::from("Track library").bold(),
        Span::from(" (Press 'q' to quit and arrow keys to navigate"),
    ]);
    frame.render_widget(title.centered(), top);

    render_track_table(frame, main, track_table_state, library);
}

// TODO: add ratatui popup rendering
pub fn render_error_popup() {}
