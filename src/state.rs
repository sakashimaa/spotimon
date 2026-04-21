use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

use rand::RngExt;
use ratatui::{crossterm::event::KeyCode, widgets::TableState};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use serde::{Deserialize, Serialize};

use crate::{config::AppConfig, track_library::TrackLibrary};

pub struct App {
    pub library: TrackLibrary,
    pub table_state: TableState,
    pub playlist_table_state: TableState,
    pub playback: PlaybackState,
    pub input_state: InputState,
    pub view_mode: ViewMode,
    pub cover_protocol: Option<StatefulProtocol>,
    pub picker: Picker,
    pub sort_state: SortState,
    pub playlist_manager: PlaylistManager,
    pub status_message: Option<(String, Instant)>,
}

#[derive(Serialize, Deserialize)]
pub struct Playlist {
    pub tracks: Vec<PathBuf>,
}

pub struct PlaylistManager {
    pub playlists: BTreeMap<String, Playlist>,
    pub path: PathBuf,
}

#[allow(unused)]
pub struct PlaybackState {
    pub current_track: Option<usize>,
    pub started_at: Option<Instant>,
    pub position: Duration,
    pub volume_level: f32,
    pub is_random_shuffle: bool,
    pub lyrics: Option<String>,
    pub lyrics_scroll: u16,
    pub prev_volume: f32,
    pub paused: bool,
    pub repeat: bool,
    pub queue: Vec<usize>,
}

#[allow(unused)]
pub struct InputState {
    pub mode: InputMode,
    pub search_query: String,
    pub filtered_indices: Option<Vec<usize>>,
    pub pending_track: Option<usize>,
}

#[allow(unused)]
pub struct SortState {
    pub field: SortField,
    pub order: SortOrder,
}

#[allow(unused)]
#[derive(PartialEq)]
pub enum ViewMode {
    Library,
    Lyrics,
    Cheatsheet,
    Queue,
    Playlists,
    PlaylistView(String),
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Search,
    CreatePlaylist,
    AddToPlaylist,
}

#[derive(Clone, Copy, PartialEq)]
#[allow(unused)]
pub enum SortField {
    Title,
    Artist,
    Album,
    Duration,
}

#[derive(Clone, Copy, PartialEq)]
#[allow(unused)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[allow(unused)]
pub enum Action {
    None,
    Quit,
    Play(usize),
    Stop,
    Pause,
    Resume,
    SeekForward(Duration),
    SeekBackward(Duration),
    SetVolume(f32),
    NextTrack,
    PrevTrack,
    NavigateUp,
    NavigateDown,
    ToggleShuffle,
    ToggleInputMode(InputMode),
    FetchLyrics(usize),
    ToggleViewMode(ViewMode),
    Sort(SortField),
    ToggleMute,
    ToggleRepeat,
    AddToQueue(usize),
    CreatePlaylist(String),
    AddToPlaylist(String),
    DeleteFromPlaylist(String),
}

impl PlaylistManager {
    pub fn new() -> Self {
        let path = dirs::config_dir()
            .unwrap()
            .join("spotimon")
            .join("playlists.toml");

        Self {
            playlists: fs::read_to_string(&path)
                .map(|s| toml::from_str::<BTreeMap<String, Playlist>>(&s).unwrap_or_default())
                .unwrap_or_default(),
            path,
        }
    }

    pub fn save(&self) {
        let _ = fs::write(
            &self.path,
            toml::to_string(&self.playlists)
                .unwrap_or_default()
                .as_bytes(),
        );
    }
}

impl App {
    pub fn new(
        library: TrackLibrary,
        table_state: TableState,
        playlist_table_state: TableState,
        playback: PlaybackState,
        picker: Picker,
    ) -> Self {
        Self {
            library,
            table_state,
            playlist_table_state,
            playback,
            input_state: InputState {
                mode: InputMode::Normal,
                search_query: String::new(),
                filtered_indices: None,
                pending_track: None,
            },
            view_mode: ViewMode::Library,
            cover_protocol: None,
            picker,
            sort_state: SortState {
                field: SortField::Title,
                order: SortOrder::Asc,
            },
            playlist_manager: PlaylistManager::new(),
            status_message: None,
        }
    }

    pub fn play_track(&mut self, idx: usize) {
        self.playback.current_track = Some(idx);
    }

    pub fn next_track_idx(&self) -> usize {
        if let Some(&idx) = self.playback.queue.first() {
            return idx;
        }

        if self.playback.repeat
            && let Some(idx) = self.playback.current_track
        {
            return idx;
        }

        if self.playback.is_random_shuffle {
            let mut rng = rand::rng();
            let mut next = rng.random_range(0..self.library.tracks.len());
            while Some(next) == self.playback.current_track && self.library.tracks.len() > 1 {
                next = rng.random_range(0..self.library.tracks.len());
            }
            return next;
        }
        match self.playback.current_track {
            Some(idx) if idx + 1 < self.library.tracks.len() => idx + 1,
            _ => 0,
        }
    }

    pub fn prev_track_idx(&self) -> usize {
        match self.playback.current_track {
            Some(idx) if idx > 0 => idx - 1,
            _ => self.library.tracks.len() - 1,
        }
    }

    pub fn update_filter(&mut self) {
        if self.input_state.search_query.is_empty() {
            self.input_state.filtered_indices = None;
            return;
        }

        let query = self.input_state.search_query.to_lowercase();
        self.input_state.filtered_indices = Some(
            self.library
                .tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| {
                    t.title.to_lowercase().contains(&query)
                        || t.artist.to_lowercase().contains(&query)
                        || t.album.to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect(),
        );

        self.table_state.select_first();
    }

    pub fn handle_search_mode(&mut self, key_code: KeyCode) -> Action {
        match key_code {
            KeyCode::Esc => {
                self.input_state.mode = InputMode::Normal;
                self.input_state.search_query.clear();
                self.input_state.filtered_indices = None;

                Action::None
            }
            KeyCode::Char(c) => {
                self.input_state.search_query.push(c);
                self.update_filter();

                Action::None
            }
            KeyCode::Backspace => {
                self.input_state.search_query.pop();

                Action::None
            }
            KeyCode::Enter => {
                self.input_state.mode = InputMode::Normal;
                self.update_filter();

                Action::None
            }
            _ => Action::None,
        }
    }

    pub fn selected_library_idx(&self) -> Option<usize> {
        if let Some(selected) = self.table_state.selected() {
            return Some(
                self.input_state
                    .filtered_indices
                    .as_ref()
                    .map(|indices| indices[selected])
                    .unwrap_or(selected),
            );
        }

        None
    }

    pub fn handle_normal_mode(&mut self, key_code: KeyCode, config: &AppConfig) -> Action {
        match key_code {
            KeyCode::Esc | KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('l') => {
                Action::SeekForward(Duration::from_secs(config.skip_interval_secs))
            }
            KeyCode::Char('h') => {
                Action::SeekBackward(Duration::from_secs(config.skip_interval_secs))
            }
            KeyCode::Char('j') | KeyCode::Down => match self.view_mode {
                ViewMode::Lyrics => {
                    self.playback.lyrics_scroll += 1;
                    Action::None
                }
                ViewMode::Playlists | ViewMode::PlaylistView(_) => {
                    self.playlist_table_state.select_next();
                    Action::None
                }

                _ => Action::NavigateDown,
            },
            KeyCode::Char('k') | KeyCode::Up => match self.view_mode {
                ViewMode::Lyrics => {
                    self.playback.lyrics_scroll = self.playback.lyrics_scroll.saturating_sub(1);
                    Action::None
                }
                ViewMode::Playlists | ViewMode::PlaylistView(_) => {
                    self.playlist_table_state.select_previous();
                    Action::None
                }

                _ => Action::NavigateUp,
            },
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.playback.volume_level = (self.playback.volume_level + 0.05).min(1.0);
                Action::SetVolume(self.playback.volume_level)
            }
            KeyCode::Char('-') => {
                self.playback.volume_level = (self.playback.volume_level - 0.05).max(0.0);
                Action::SetVolume(self.playback.volume_level)
            }
            KeyCode::Char('n') | KeyCode::Char('N') => Action::NextTrack,
            KeyCode::Char('p') | KeyCode::Char('P') => Action::PrevTrack,
            KeyCode::Char(' ') => Action::Pause,
            KeyCode::Char('s') => Action::ToggleShuffle,
            KeyCode::Char('L') => Action::ToggleViewMode(ViewMode::Lyrics),
            KeyCode::Backspace => Action::ToggleViewMode(ViewMode::Library),
            KeyCode::Enter => match &self.view_mode {
                ViewMode::Playlists => {
                    if let Some(selected) = self.playlist_table_state.selected()
                        && let Some(playlist_name) =
                            self.playlist_manager.playlists.keys().nth(selected)
                    {
                        return Action::ToggleViewMode(ViewMode::PlaylistView(
                            playlist_name.clone(),
                        ));
                    }

                    Action::None
                }
                ViewMode::PlaylistView(name) => {
                    let name = name.clone();
                    if let Some(selected) = self.playlist_table_state.selected()
                        && let Some(playlist) = self.playlist_manager.playlists.get(&name)
                        && let Some(track_path) = playlist.tracks.get(selected)
                        && let Some(track_idx) = self
                            .library
                            .tracks
                            .iter()
                            .position(|t| t.path == *track_path)
                    {
                        let remaining: Vec<usize> = playlist
                            .tracks
                            .iter()
                            .skip(selected + 1)
                            .filter_map(|p| self.library.tracks.iter().position(|t| t.path == *p))
                            .collect();

                        self.playback.queue.clear();
                        self.playback.queue.extend(remaining);

                        return Action::Play(track_idx);
                    }

                    Action::None
                }
                _ => {
                    self.playback.lyrics_scroll = 0;
                    if let Some(real_idx) = self.selected_library_idx() {
                        self.playback.queue.clear();
                        Action::Play(real_idx)
                    } else {
                        Action::None
                    }
                }
            },
            KeyCode::Char('/') => Action::ToggleInputMode(InputMode::Search),
            KeyCode::Char('?') => Action::ToggleViewMode(ViewMode::Cheatsheet),
            KeyCode::Char('1') => Action::Sort(SortField::Title),
            KeyCode::Char('2') => Action::Sort(SortField::Artist),
            KeyCode::Char('3') => Action::Sort(SortField::Album),
            KeyCode::Char('4') => Action::Sort(SortField::Duration),
            KeyCode::Char('m') | KeyCode::Char('M') => Action::ToggleMute,
            KeyCode::Char('r') | KeyCode::Char('R') => Action::ToggleRepeat,
            KeyCode::Char('a') | KeyCode::Char('A') => {
                if let Some(real_idx) = self.selected_library_idx() {
                    Action::AddToQueue(real_idx)
                } else {
                    Action::None
                }
            }
            KeyCode::Char('z') | KeyCode::Char('Z') => Action::ToggleViewMode(ViewMode::Queue),
            KeyCode::Char('c') | KeyCode::Char('C') => {
                Action::ToggleInputMode(InputMode::CreatePlaylist)
            }
            KeyCode::Char('t') | KeyCode::Char('T') => Action::ToggleViewMode(ViewMode::Playlists),
            KeyCode::Char(':') => {
                if let Some(real_idx) = self.selected_library_idx() {
                    self.input_state.pending_track = Some(real_idx);
                    Action::ToggleInputMode(InputMode::AddToPlaylist)
                } else {
                    Action::None
                }
            }
            KeyCode::Char('d') => match &self.view_mode {
                ViewMode::PlaylistView(name) => {
                    if let Some(track_idx) = self.playlist_table_state.selected() {
                        self.input_state.pending_track = Some(track_idx);
                        return Action::DeleteFromPlaylist(name.clone());
                    }
                    Action::None
                }
                _ => Action::None,
            },
            _ => Action::None,
        }
    }

    pub fn apply_sort(&mut self) {
        let field = self.sort_state.field;
        let order = self.sort_state.order;

        self.library.tracks.sort_by(|a, b| {
            let cmp = match field {
                SortField::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                SortField::Artist => a.artist.to_lowercase().cmp(&b.artist.to_lowercase()),
                SortField::Album => a.album.to_lowercase().cmp(&b.album.to_lowercase()),
                SortField::Duration => a.duration.cmp(&b.duration),
            };

            if order == SortOrder::Desc {
                cmp.reverse()
            } else {
                cmp
            }
        })
    }

    pub fn handle_create_playlist(&mut self, key_code: KeyCode) -> Action {
        match key_code {
            KeyCode::Esc => {
                self.input_state.mode = InputMode::Normal;
                self.input_state.search_query.clear();
                Action::None
            }
            KeyCode::Char(c) => {
                self.input_state.search_query.push(c);
                Action::None
            }
            KeyCode::Backspace => {
                self.input_state.search_query.pop();
                Action::None
            }
            KeyCode::Enter => {
                let name = self.input_state.search_query.clone();
                self.input_state.search_query.clear();
                self.input_state.mode = InputMode::Normal;
                Action::CreatePlaylist(name)
            }
            _ => Action::None,
        }
    }

    pub fn handle_add_to_playlist(&mut self, key_code: KeyCode) -> Action {
        match key_code {
            KeyCode::Esc => {
                self.input_state.mode = InputMode::Normal;
                Action::None
            }
            KeyCode::Char('j') => {
                self.playlist_table_state.select_next();
                Action::None
            }
            KeyCode::Char('k') => {
                self.playlist_table_state.select_previous();
                Action::None
            }
            KeyCode::Enter => {
                if let Some(selected) = self.playlist_table_state.selected()
                    && let Some(playlist_name) =
                        self.playlist_manager.playlists.iter().nth(selected)
                {
                    return Action::AddToPlaylist(playlist_name.0.to_string());
                }

                Action::None
            }
            _ => Action::None,
        }
    }
}

impl PlaybackState {
    pub fn new(config: &AppConfig) -> Self {
        let config_vol = config.device.volume as f32 / 100.0;

        Self {
            current_track: None,
            started_at: None,
            paused: false,
            position: Duration::new(0, 0),
            volume_level: config_vol,
            is_random_shuffle: false,
            lyrics: None,
            lyrics_scroll: 0,
            prev_volume: config_vol,
            repeat: false,
            queue: Vec::new(),
        }
    }
}
