use std::{
    fs::{self, File},
    io::BufReader,
    time::Duration,
};

use ratatui::{
    crossterm::event::{self, KeyCode},
    widgets::TableState,
};
use rodio::Decoder;

use crate::state::{App, InputMode, PlaybackState};

mod config;
mod state;
mod track_library;
mod ui;

fn get_track_source(idx: usize, app_state: &App) -> Option<Decoder<BufReader<File>>> {
    if let Some(track) = app_state.library.tracks.get(idx)
        && let Ok(file) = File::open(&track.path)
        && let Ok(source) = Decoder::try_from(file)
    {
        return Some(source);
    }

    None
}

fn main() -> color_eyre::Result<()> {
    let config_dir = dirs::config_dir().expect("No config dir").join("spotimon");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create config dir");
    }

    let config_path = config_dir.join("config.toml");

    let config_contents: config::AppConfig = match fs::read_to_string(&config_path) {
        Ok(r) => toml::from_str(&r).unwrap_or_else(|_| config::write_default_config(&config_path)),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => config::write_default_config(&config_path),
            e => {
                panic!("Unknown error: {e}");
            }
        },
    };

    if !config_contents.music_folder.exists() {
        fs::create_dir_all(&config_contents.music_folder).expect("Failed to create music dir");
    }

    color_eyre::install()?;
    let library = track_library::TrackLibrary::new(&config_contents.music_folder);

    let mut terminal = ratatui::init();
    let mut track_table_state = TableState::default();
    track_table_state.select_first();
    track_table_state.select_first_column();

    let handle =
        rodio::DeviceSinkBuilder::open_default_sink().expect("Failed to open audio stream");
    let player = rodio::Player::connect_new(handle.mixer());
    let mut app_state = App::new(library, track_table_state, PlaybackState::default());

    loop {
        app_state.playback.position = player.get_pos();

        if player.empty() && app_state.playback.current_track.is_some() {
            let next_idx = app_state.next_track_idx();

            if let Some(source) = get_track_source(next_idx, &app_state) {
                player.append(source);
                app_state.play_track(next_idx);
            }
        }

        terminal.draw(|frame| {
            ui::render::render(frame, &mut app_state);
        })?;

        if event::poll(Duration::from_millis(100))?
            && let Some(key) = event::read()?.as_key_press_event()
        {
            match app_state.input_state.mode {
                InputMode::Normal => match key.code {
                    KeyCode::Esc => {
                        if app_state.input_state.filtered_indices.is_some() {
                            app_state.input_state.search_query.clear();
                            app_state.input_state.filtered_indices = None;
                        }
                    }
                    KeyCode::Char('l') => {
                        let new_pos = player.get_pos()
                            + Duration::from_secs(config_contents.skip_interval_secs);
                        let _ = player.try_seek(new_pos);
                    }
                    KeyCode::Char('h') => {
                        let new_pos = player
                            .get_pos()
                            .checked_sub(Duration::from_secs(config_contents.skip_interval_secs))
                            .unwrap_or(Duration::ZERO);
                        let _ = player.try_seek(new_pos);
                    }
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => app_state.table_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => app_state.table_state.select_previous(),
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        app_state.playback.volume_level =
                            (app_state.playback.volume_level + 0.05).min(1.0);
                        player.set_volume(app_state.playback.volume_level);
                    }
                    KeyCode::Char('-') => {
                        app_state.playback.volume_level =
                            (app_state.playback.volume_level - 0.05).max(0.0);
                        player.set_volume(app_state.playback.volume_level);
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        let next_idx = app_state.next_track_idx();

                        if let Some(source) = get_track_source(next_idx, &app_state) {
                            player.stop();
                            player.append(source);
                            app_state.play_track(next_idx);
                        }
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        let prev_idx = app_state.prev_track_idx();

                        if let Some(source) = get_track_source(prev_idx, &app_state) {
                            player.stop();
                            player.append(source);
                            app_state.play_track(prev_idx);
                        }
                    }
                    KeyCode::Char(' ') => {
                        if player.is_paused() {
                            player.play();
                        } else {
                            player.pause();
                        }
                    }
                    KeyCode::Char('s') => {
                        app_state.playback.is_random_shuffle =
                            !app_state.playback.is_random_shuffle;
                    }
                    KeyCode::Enter => {
                        if let Some(selected) = app_state.table_state.selected() {
                            let real_idx = app_state
                                .input_state
                                .filtered_indices
                                .as_ref()
                                .map(|indices| indices[selected])
                                .unwrap_or(selected);

                            if let Some(source) = get_track_source(real_idx, &app_state) {
                                player.stop();
                                player.append(source);
                                app_state.playback.current_track = Some(real_idx);
                            }
                        }
                    }
                    KeyCode::Char('/') => {
                        app_state.input_state.mode = InputMode::Search;
                    }
                    _ => {}
                },
                InputMode::Search => match key.code {
                    KeyCode::Esc => {
                        app_state.input_state.mode = InputMode::Normal;
                        app_state.input_state.search_query.clear();
                        app_state.input_state.filtered_indices = None;
                    }
                    KeyCode::Char(c) => {
                        app_state.input_state.search_query.push(c);
                        app_state.update_filter();
                    }
                    KeyCode::Backspace => {
                        app_state.input_state.search_query.pop();
                    }
                    KeyCode::Enter => {
                        app_state.input_state.mode = InputMode::Normal;
                        app_state.update_filter();
                    }
                    _ => {}
                },
            }
        }
    }
    ratatui::restore();

    Ok(())
}
