use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::prelude::ItemKey;
use ratatui::{
    Frame,
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Row, Table, TableState},
};
use rodio::Decoder;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
struct AppConfig {
    device: DeviceConfig,
    music_folder: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct DeviceConfig {
    volume: u32,
}

struct TrackLibrary {
    tracks: Vec<Track>,
}

impl TrackLibrary {
    fn new(music_folder: &PathBuf) -> Self {
        let tracks: Vec<_> = WalkDir::new(music_folder)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| matches!(ext, "mp3" | "flac" | "ogg"))
            })
            .filter_map(|f| {
                let tagged = lofty::read_from_path(f.path()).ok()?;
                let tag = tagged.primary_tag().or(tagged.first_tag())?;

                Some(Track {
                    title: tag
                        .get_string(ItemKey::TrackTitle)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| {
                            f.path()
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        }),
                    artist: tag
                        .get_string(ItemKey::TrackArtist)
                        .filter(|s| !s.is_empty())
                        .unwrap_or("Unknown")
                        .to_string(),
                    album: tag
                        .get_string(ItemKey::AlbumTitle)
                        .filter(|s| !s.is_empty())
                        .unwrap_or("Unknown")
                        .to_string(),
                    duration: tagged.properties().duration(),
                    path: f.path().to_path_buf(),
                })
            })
            .collect();

        Self { tracks }
    }
}

struct Track {
    title: String,
    artist: String,
    album: String,
    duration: std::time::Duration,
    #[allow(unused)]
    path: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            device: DeviceConfig { volume: 50 },
            music_folder: dirs::home_dir().expect("No home dir").join("Music"),
        }
    }
}

fn write_default_config(path: &Path) -> AppConfig {
    let default_conf = AppConfig::default();
    let toml_str = toml::to_string(&default_conf).expect("Failed to serialize");
    fs::write(path, &toml_str).expect("Failed to write");

    default_conf
}

fn render_track_table(
    frame: &mut Frame,
    area: Rect,
    table_state: &mut TableState,
    track_library: &TrackLibrary,
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

fn render(frame: &mut Frame, library: &TrackLibrary, track_table_state: &mut TableState) {
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
fn render_error_popup() {}

fn main() -> color_eyre::Result<()> {
    let config_dir = dirs::config_dir().expect("No config dir").join("spotimon");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create config dir");
    }

    let config_path = config_dir.join("config.toml");

    let config_contents: AppConfig = match fs::read_to_string(&config_path) {
        Ok(r) => toml::from_str(&r).unwrap_or_else(|_| write_default_config(&config_path)),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => write_default_config(&config_path),
            e => {
                panic!("Unknown error: {e}");
            }
        },
    };

    if !config_contents.music_folder.exists() {
        fs::create_dir_all(&config_contents.music_folder).expect("Failed to create music dir");
    }

    color_eyre::install()?;
    let library = TrackLibrary::new(&config_contents.music_folder);

    let mut terminal = ratatui::init();
    let mut track_table_state = TableState::default();
    track_table_state.select_first();
    track_table_state.select_first_column();

    let handle =
        rodio::DeviceSinkBuilder::open_default_sink().expect("Failed to open audio stream");
    let player = rodio::Player::connect_new(handle.mixer());

    loop {
        terminal.draw(|frame| {
            render(frame, &library, &mut track_table_state);
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
                KeyCode::Char(' ') => {
                    if player.is_paused() {
                        player.play()
                    } else {
                        player.pause()
                    }
                }
                _ => {}
            }
        }
    }
    ratatui::restore();

    Ok(())
}
