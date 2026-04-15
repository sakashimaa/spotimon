use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Gauge, Paragraph, Row, Table},
};

use crate::state::{App, InputMode, ViewMode};

fn render_volume_level(frame: &mut Frame, area: Rect, app_state: &App) {
    let volume_humanized = (app_state.playback.volume_level * 100.0) as u16;
    let text = format!("Vol: %{volume_humanized}");
    let gauge = Gauge::default()
        .gauge_style(Style::new().green().on_black())
        .ratio(app_state.playback.volume_level as f64)
        .label(text);
    frame.render_widget(gauge, area);
}

fn render_track_progress(frame: &mut Frame, area: Rect, app_state: &App) {
    if let Some(idx) = app_state.playback.current_track
        && let Some(track) = app_state.library.tracks.get(idx)
    {
        let progress = app_state.playback.position.as_secs_f64() / track.duration.as_secs_f64();

        let pos_mins = app_state.playback.position.as_secs() / 60;
        let pos_secs = app_state.playback.position.as_secs() % 60;
        let dur_mins = track.duration.as_secs() / 60;
        let dur_secs = track.duration.as_secs() % 60;

        let label = format!(
            "{}:{:02} / {}:{:02}",
            pos_mins, pos_secs, dur_mins, dur_secs
        );

        let gauge = Gauge::default()
            .style(Modifier::BOLD)
            .gauge_style(Style::new().cyan().on_black())
            .ratio(progress.min(1.0))
            .label(label);

        frame.render_widget(gauge, area);
    }
}

fn render_track_creds(frame: &mut Frame, area: Rect, app_state: &App) {
    if let Some(track_idx) = app_state.playback.current_track {
        let curr_track = &app_state.library.tracks[track_idx];
        let creds = format!("{} - {}", curr_track.artist, curr_track.title);

        let cred_paragraph = Paragraph::new(creds);
        frame.render_widget(cred_paragraph, area);
    }
}

fn render_track_table(frame: &mut Frame, area: Rect, app_state: &mut App) {
    let header = Row::new(["Title", "Artist", "Album", "Duration"])
        .style(Style::new().bold())
        .bottom_margin(1);

    let indices: Vec<usize> = app_state
        .input_state
        .filtered_indices
        .clone()
        .unwrap_or_else(|| (0..app_state.library.tracks.len()).collect());

    let rows: Vec<Row> = indices
        .iter()
        .filter_map(|&i| app_state.library.tracks.get(i))
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

    frame.render_stateful_widget(table, area, &mut app_state.table_state);
}

pub fn render_lyrics(frame: &mut Frame, area: Rect, app_state: &App) {
    let lyrics = app_state
        .playback
        .lyrics
        .as_deref()
        .unwrap_or("No lyrics available.");

    let paragraph = Paragraph::new(lyrics).scroll((app_state.playback.lyrics_scroll, 0));
    frame.render_widget(paragraph, area);
}

pub fn render(frame: &mut Frame, app_state: &mut App) {
    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .spacing(1);
    let [top, main, creds, track_bar] = frame.area().layout(&layout);

    let bottom_layout =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(10)]).split(track_bar);

    let shuffle_indicator = if app_state.playback.is_random_shuffle {
        Span::from(" ").style(Style::new().green())
    } else {
        Span::from(" ")
    };

    let centered_label = if app_state.input_state.mode == InputMode::Search {
        Line::from(format!("/{}", app_state.input_state.search_query.clone()))
    } else {
        Line::from_iter([
            Span::from("Track library").bold(),
            shuffle_indicator,
            Span::from(" (q: quit, j/k: nav, s: shuffle)"),
        ])
    };
    frame.render_widget(centered_label.centered(), top);

    match app_state.view_mode {
        ViewMode::Library => {
            render_track_table(frame, main, app_state);
        }
        ViewMode::Lyrics => {
            render_lyrics(frame, main, app_state);
        }
    }

    render_track_progress(frame, bottom_layout[0], app_state);
    render_volume_level(frame, bottom_layout[1], app_state);
    render_track_creds(frame, creds, app_state);
}

// TODO: add ratatui popup rendering
pub fn render_error_popup() {}
