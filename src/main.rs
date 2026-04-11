use std::fs::{self, File};

use ratatui::{
    crossterm::event::{self, KeyCode},
    widgets::TableState,
};
use rodio::Decoder;

mod config;
mod track_library;
mod ui;

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

    loop {
        terminal.draw(|frame| {
            ui::render::render(frame, &library, &mut track_table_state);
        })?;

        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('j') | KeyCode::Down => track_table_state.select_next(),
                KeyCode::Char('k') | KeyCode::Up => track_table_state.select_previous(),
                KeyCode::Enter => {
                    if let Some(idx) = track_table_state.selected()
                        && let Some(track) = library.tracks.get(idx)
                    {
                        eprintln!("Playing: {}", track.title);
                        let file = match File::open(&track.path) {
                            Ok(f) => f,
                            Err(_) => continue,
                        };
                        let source = match Decoder::try_from(file) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        handle.mixer().add(source);
                    }
                }
                _ => {}
            }
        }
    }
    ratatui::restore();

    Ok(())
}
