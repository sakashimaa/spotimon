use std::time::{Duration, Instant};

use rand::RngExt;
use ratatui::{crossterm::event::KeyCode, widgets::TableState};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

use crate::{config::AppConfig, track_library::TrackLibrary};

pub struct App {
    pub library: TrackLibrary,
    pub table_state: TableState,
    pub playback: PlaybackState,
    pub input_state: InputState,
    pub view_mode: ViewMode,
    pub cover_protocol: Option<StatefulProtocol>,
    pub picker: Picker,
    pub sort_state: SortState,
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
}

#[allow(unused)]
pub struct InputState {
    pub mode: InputMode,
    pub search_query: String,
    pub filtered_indices: Option<Vec<usize>>,
}

#[allow(unused)]
pub enum ViewMode {
    Library,
    Lyrics,
    Cheatsheet,
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Search,
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
pub struct SortState {
    pub field: SortField,
    pub order: SortOrder,
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
}

impl App {
    pub fn new(
        library: TrackLibrary,
        table_state: TableState,
        playback: PlaybackState,
        picker: Picker,
    ) -> Self {
        Self {
            library,
            table_state,
            playback,
            input_state: InputState {
                mode: InputMode::Normal,
                search_query: String::new(),
                filtered_indices: None,
            },
            view_mode: ViewMode::Library,
            cover_protocol: None,
            picker,
            sort_state: SortState {
                field: SortField::Title,
                order: SortOrder::Asc,
            },
        }
    }

    pub fn play_track(&mut self, idx: usize) {
        self.playback.current_track = Some(idx);
    }

    pub fn next_track_idx(&self) -> usize {
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
                _ => Action::NavigateDown,
            },
            KeyCode::Char('k') | KeyCode::Up => match self.view_mode {
                ViewMode::Lyrics => {
                    self.playback.lyrics_scroll = self.playback.lyrics_scroll.saturating_sub(1);
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
            KeyCode::Enter => {
                self.playback.lyrics_scroll = 0;
                if let Some(selected) = self.table_state.selected() {
                    let real_idx = self
                        .input_state
                        .filtered_indices
                        .as_ref()
                        .map(|indices| indices[selected])
                        .unwrap_or(selected);

                    Action::Play(real_idx)
                } else {
                    Action::None
                }
            }
            KeyCode::Char('/') => Action::ToggleInputMode(InputMode::Search),
            KeyCode::Char('?') => Action::ToggleViewMode(ViewMode::Cheatsheet),
            KeyCode::Char('1') => Action::Sort(SortField::Title),
            KeyCode::Char('2') => Action::Sort(SortField::Artist),
            KeyCode::Char('3') => Action::Sort(SortField::Album),
            KeyCode::Char('4') => Action::Sort(SortField::Duration),
            KeyCode::Char('m') | KeyCode::Char('M') => Action::ToggleMute,
            KeyCode::Char('r') | KeyCode::Char('R') => Action::ToggleRepeat,
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
        }
    }
}
