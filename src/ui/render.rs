use std::time::Duration;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, Gauge, Paragraph, Row, Table},
};
use ratatui_image::StatefulImage;

use crate::{
    config::AppConfig,
    state::{App, InputMode, SortField, ViewMode},
    track_library::Track,
    utils::{centered_rect, sort_indicator},
};

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

pub fn render_playlists_table(frame: &mut Frame, area: Rect, app_state: &mut App) {
    let header = Row::new(["Name"])
        .style(Style::new().bold())
        .bottom_margin(1);

    let rows: Vec<Row> = app_state
        .playlist_manager
        .playlists
        .iter()
        .map(|p| Row::new([p.0.clone()]))
        .collect();

    let widths = [Constraint::Percentage(100)];

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(1)
        .style(Color::Blue)
        .row_highlight_style(Style::new().on_black().bold())
        .column_highlight_style(Color::Gray);

    frame.render_stateful_widget(table, area, &mut app_state.playlist_table_state);
}

pub fn render_add_to_playlist_popup(frame: &mut Frame, area: Rect, app_state: &mut App) {
    let block = Block::bordered()
        .title(" Add to playlist ")
        .border_style(Style::new().cyan());
    let inner = block.inner(area);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    render_playlists_table(frame, inner, app_state);
}

pub fn render_playlist_tracks(
    frame: &mut Frame,
    area: Rect,
    playlist_name: &str,
    app_state: &mut App,
) {
    if let Some(playlist) = app_state.playlist_manager.playlists.get(playlist_name) {
        let lib_tracks: Vec<Option<(usize, &Track)>> = playlist
            .tracks
            .iter()
            .map(|tr| {
                app_state
                    .library
                    .tracks
                    .iter()
                    .enumerate()
                    .find(|(_, t)| t.path == *tr)
            })
            .collect();

        let header = Row::new([
            sort_indicator("Title", SortField::Title, &app_state.sort_state),
            sort_indicator("Artist", SortField::Artist, &app_state.sort_state),
            sort_indicator("Album", SortField::Album, &app_state.sort_state),
            sort_indicator("Duration", SortField::Duration, &app_state.sort_state),
        ])
        .style(Style::new().bold())
        .bottom_margin(1);

        let current = app_state.playback.current_track;
        let rows: Vec<Row> = lib_tracks
            .iter()
            .filter_map(|o| o.as_ref())
            .map(|&(i, t)| track_row(t, i, current))
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

        frame.render_stateful_widget(table, area, &mut app_state.playlist_table_state);
    } else {
        let not_found_text = Paragraph::new("Not found.");
        let centered_area = centered_rect(40, 50, frame.area());
        frame.render_widget(not_found_text, centered_area);
    }
}

fn track_row(track: &Track, idx: usize, current: Option<usize>) -> Row<'_> {
    let mins = track.duration.as_secs() / 60;
    let secs = track.duration.as_secs() % 60;

    let row = Row::new([
        track.title.clone(),
        track.artist.clone(),
        track.album.clone(),
        format!("{}:{:02}", mins, secs),
    ]);

    if let Some(curr) = current
        && curr == idx
    {
        row.style(Style::new().red())
    } else {
        row
    }
}

fn render_track_table(frame: &mut Frame, area: Rect, app_state: &mut App) {
    let header = Row::new([
        sort_indicator("Title", SortField::Title, &app_state.sort_state),
        sort_indicator("Artist", SortField::Artist, &app_state.sort_state),
        sort_indicator("Album", SortField::Album, &app_state.sort_state),
        sort_indicator("Duration", SortField::Duration, &app_state.sort_state),
    ])
    .style(Style::new().bold())
    .bottom_margin(1);

    let indices: Vec<usize> = app_state
        .input_state
        .filtered_indices
        .clone()
        .unwrap_or_else(|| (0..app_state.library.tracks.len()).collect());

    let current = app_state.playback.current_track;
    let rows: Vec<Row> = indices
        .iter()
        .filter_map(|&i| app_state.library.tracks.get(i).map(|t| (i, t)))
        .map(|(i, t)| track_row(t, i, current))
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

fn render_track_creds(frame: &mut Frame, area: Rect, app_state: &App) {
    if let Some(track_idx) = app_state.playback.current_track {
        let curr_track = &app_state.library.tracks[track_idx];

        let creds = Line::from_iter([
            Span::from(curr_track.artist.as_str()).bold(),
            Span::from(" - "),
            Span::from(curr_track.title.as_str()),
        ]);

        let cred_paragraph = Paragraph::new(creds);
        frame.render_widget(cred_paragraph, area);
    }
}

fn render_lyrics(frame: &mut Frame, area: Rect, app_state: &App) {
    let lyrics = app_state
        .playback
        .lyrics
        .as_deref()
        .unwrap_or("No lyrics available.");

    let paragraph = Paragraph::new(lyrics).scroll((app_state.playback.lyrics_scroll, 0));
    frame.render_widget(paragraph, area);
}

fn render_cover(frame: &mut Frame, area: Rect, app_state: &mut App) {
    if let Some(protocol) = &mut app_state.cover_protocol {
        let image = StatefulImage::default();
        frame.render_stateful_widget(image, area, protocol);
    }
}

fn render_cheatsheet(frame: &mut Frame) {
    let lines = vec![
        Line::from(vec![
            Span::from("j/k").bold(),
            Span::from("  navigate down/up"),
        ]),
        Line::from(vec![Span::from("Enter").bold(), Span::from("  play track")]),
        Line::from(vec![Span::from("q").bold(), Span::from("  quit")]),
        Line::from(vec![Span::from("?").bold(), Span::from("  toggle help")]),
        Line::from(vec![
            Span::from("Backspace").bold(),
            Span::from("  go to library"),
        ]),
        Line::from(vec![Span::from("m/M").bold(), Span::from("  toggle mute")]),
        Line::from(vec![
            Span::from("r/R").bold(),
            Span::from("  toggle repeat"),
        ]),
        Line::from(vec![Span::from("a/A").bold(), Span::from("  add to queue")]),
        Line::from(vec![Span::from("z/Z").bold(), Span::from("  view queue")]),
        Line::from(vec![
            Span::from("t/T").bold(),
            Span::from("  view playlists"),
        ]),
        Line::from(vec![
            Span::from("c/C").bold(),
            Span::from("  create playlist"),
        ]),
        Line::from(vec![
            Span::from(":").bold(),
            Span::from("  add to playlist"),
        ]),
        Line::from(vec![
            Span::from("d").bold(),
            Span::from("  delete from playlist"),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::bordered()
            .title(" Keybinds ")
            .border_style(Style::new().cyan()),
    );

    let area = centered_rect(40, 50, frame.area());
    frame.render_widget(paragraph, area);
}

fn render_queue(frame: &mut Frame, area: Rect, app_state: &App) {
    if app_state.playback.queue.is_empty() {
        let empty = Paragraph::new("Queue is empty. Press 'a' to add tracks")
            .style(Style::new().dark_gray());
        frame.render_widget(empty, area);
        return;
    }

    let rows: Vec<Row> = app_state
        .playback
        .queue
        .iter()
        .enumerate()
        .filter_map(|(pos, &idx)| {
            app_state.library.tracks.get(idx).map(|t| {
                let mins = t.duration.as_secs() / 60;
                let secs = t.duration.as_secs() % 60;
                Row::new([
                    format!("{}", pos + 1),
                    t.title.clone(),
                    t.artist.clone(),
                    format!("{}:{:02}", mins, secs),
                ])
            })
        })
        .collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Percentage(40),
        Constraint::Percentage(40),
        Constraint::Length(6),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(["#", "Title", "Artist", "Duration"])
                .style(Style::new().bold())
                .bottom_margin(1),
        )
        .block(
            Block::bordered()
                .title(" Queue ")
                .border_style(Style::new().cyan()),
        )
        .style(Color::Blue);

    frame.render_widget(table, area);
}

pub fn render(frame: &mut Frame, app_state: &mut App, app_config: &AppConfig) {
    let has_cover = app_state.cover_protocol.is_some();
    let cover_height: u16 = if has_cover { 6 } else { 0 };

    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(cover_height + 2),
        Constraint::Length(1),
    ])
    .spacing(1);
    let [top, main, bottom_info, track_bar] = frame.area().layout(&layout);

    let bottom_panel = Layout::vertical([
        Constraint::Length(cover_height),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .spacing(if has_cover { 1 } else { 0 })
    .split(bottom_info);

    let cover_area = bottom_panel[0];
    let creds_area = bottom_panel[1];

    let track_bar_layout =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(10)]).split(track_bar);

    let shuffle_indicator = if app_state.playback.is_random_shuffle {
        Span::from(" ").style(Style::new().green())
    } else {
        Span::from(" ")
    };

    let centered_label = if app_state.input_state.mode == InputMode::Search {
        Line::from(format!("/{}", app_state.input_state.search_query.clone()))
    } else if let Some(status_message) = &app_state.status_message
        && status_message.1.elapsed() < Duration::from_secs(app_config.notify_message_live_seconds)
    {
        Line::from(status_message.0.clone())
    } else {
        let context_aware_title = match &app_state.view_mode {
            ViewMode::Queue => "Queue".to_string(),
            ViewMode::Lyrics => "Lyrics".to_string(),
            ViewMode::Playlists => "Playlists".to_string(),
            ViewMode::Library => "Track library".to_string(),
            ViewMode::Cheatsheet => "Cheatsheet".to_string(),
            ViewMode::PlaylistView(name) => {
                let tracks_quantity = app_state
                    .playlist_manager
                    .playlists
                    .iter()
                    .find(|p| p.0 == name)
                    .map(|p| p.1.tracks.len())
                    .unwrap_or_default();

                format!("{} ({} tracks)", name, tracks_quantity)
            }
        };

        Line::from_iter([
            Span::from(context_aware_title),
            shuffle_indicator,
            Span::from(" (q: quit, j/k: nav, s: shuffle, ?: cheatsheet)"),
        ])
    };
    frame.render_widget(centered_label.centered(), top);

    if app_state.input_state.mode == InputMode::CreatePlaylist {
        let popup = Paragraph::new(format!("Name: {}", app_state.input_state.search_query)).block(
            Block::bordered()
                .title(" New playlist ")
                .border_style(Style::new().cyan()),
        );
        let area = centered_rect(80, 40, frame.area());
        frame.render_widget(popup, area);

        return;
    }

    if app_state.input_state.mode == InputMode::AddToPlaylist {
        let area = centered_rect(50, 60, frame.area());
        render_add_to_playlist_popup(frame, area, app_state);

        return;
    }

    match &app_state.view_mode {
        ViewMode::Library => {
            render_track_table(frame, main, app_state);
        }
        ViewMode::Lyrics => {
            render_lyrics(frame, main, app_state);
        }
        ViewMode::Cheatsheet => {
            render_cheatsheet(frame);
        }
        ViewMode::Queue => {
            render_queue(frame, main, app_state);
        }
        ViewMode::Playlists => {
            render_playlists_table(frame, main, app_state);
        }
        ViewMode::PlaylistView(name) => {
            let name = name.clone();
            render_playlist_tracks(frame, main, &name, app_state);
        }
    }

    render_cover(frame, cover_area, app_state);
    render_track_creds(frame, creds_area, app_state);
    render_track_progress(frame, track_bar_layout[0], app_state);
    render_volume_level(frame, track_bar_layout[1], app_state);
}
