use std::{
    fs::{self},
    time::Duration,
};

use ratatui::{
    crossterm::event::{self, KeyCode},
    widgets::TableState,
};

use crate::{
    state::{App, InputMode, PlaybackState},
    utils::get_track_source,
};

mod config;
mod state;
mod track_library;
mod ui;
mod utils;

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
                InputMode::Normal => {
                    app_state.handle_normal_mode(key.code, &player, &config_contents);
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
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
