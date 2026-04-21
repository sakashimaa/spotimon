use std::{
    fs::{self},
    sync::mpsc,
    time::Duration,
};

use ratatui::{
    crossterm::event::{self},
    widgets::TableState,
};
use ratatui_image::picker::Picker;

use crate::{
    state::{Action, App, InputMode, PlaybackState},
    utils::get_track_source,
};

mod config;
mod lyrics;
mod mpris;
mod player_controller;
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

    let mut playlist_table_state = TableState::default();
    playlist_table_state.select_first();
    playlist_table_state.select_first_column();

    let handle =
        rodio::DeviceSinkBuilder::open_default_sink().expect("Failed to open audio stream");
    let player = rodio::Player::connect_new(handle.mixer());
    player.set_volume(config_contents.device.volume as f32 / 100.0);

    let picker = Picker::from_query_stdio().unwrap_or(Picker::halfblocks());

    let mut app_state = App::new(
        library,
        track_table_state,
        playlist_table_state,
        PlaybackState::new(&config_contents),
        picker,
    );

    let (lyrics_tx, lyrics_rx) = mpsc::channel::<Option<String>>();
    let (mpris_tx, mpris_rx) = mpsc::channel::<Action>();
    let mut controls = mpris::create_controls().expect("Failed to create MPRIS");
    mpris::attach_handler(&mut controls, mpris_tx);

    loop {
        app_state.playback.position = player.get_pos();

        if player.empty() && app_state.playback.current_track.is_some() {
            app_state.playback.lyrics_scroll = 0;
            let next_idx = app_state.next_track_idx();

            if let Some(source) = get_track_source(next_idx, &app_state) {
                player.append(source);
                app_state.play_track(next_idx);
                player_controller::execute(
                    Action::Play(next_idx),
                    &player,
                    &mut app_state,
                    &lyrics_tx,
                    &mut controls,
                );
            }
        }

        if let Ok(lyrics) = lyrics_rx.try_recv() {
            app_state.playback.lyrics = lyrics;
            app_state.playback.lyrics_scroll = 0;
        }

        if let Ok(action) = mpris_rx.try_recv() {
            player_controller::execute(action, &player, &mut app_state, &lyrics_tx, &mut controls);
        }

        if let Some(status_message) = &app_state.status_message
            && status_message.1.elapsed()
                > Duration::from_secs(config_contents.notify_message_live_seconds)
        {
            app_state.status_message = None
        }

        terminal.draw(|frame| {
            ui::render::render(frame, &mut app_state, &config_contents);
        })?;

        if event::poll(Duration::from_millis(16))?
            && let Some(key) = event::read()?.as_key_press_event()
        {
            let action = match app_state.input_state.mode {
                InputMode::Normal => app_state.handle_normal_mode(key.code, &config_contents),
                InputMode::Search => app_state.handle_search_mode(key.code),
                InputMode::CreatePlaylist => app_state.handle_create_playlist(key.code),
                InputMode::AddToPlaylist => app_state.handle_add_to_playlist(key.code),
            };

            if matches!(action, Action::Quit) {
                break;
            }

            player_controller::execute(action, &player, &mut app_state, &lyrics_tx, &mut controls);
        }
    }
    ratatui::restore();

    Ok(())
}
