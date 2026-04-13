use std::time::{Duration, Instant};

use rand::RngExt;
use ratatui::{crossterm::event::KeyCode, widgets::TableState};

use crate::{config::AppConfig, track_library::TrackLibrary, utils::get_track_source};

pub struct App {
    pub library: TrackLibrary,
    pub table_state: TableState,
    pub playback: PlaybackState,
    pub input_state: InputState,
}

#[allow(unused)]
pub struct PlaybackState {
    pub current_track: Option<usize>,
    pub started_at: Option<Instant>,
    pub paused: bool,
    pub position: Duration,
    pub volume_level: f32,
    pub is_random_shuffle: bool,
}

#[allow(unused)]
pub struct InputState {
    pub mode: InputMode,
    pub search_query: String,
    pub filtered_indices: Option<Vec<usize>>,
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Search,
}

impl App {
    pub fn new(library: TrackLibrary, table_state: TableState, playback: PlaybackState) -> Self {
        Self {
            library,
            table_state,
            playback,
            input_state: InputState {
                mode: InputMode::Normal,
                search_query: String::new(),
                filtered_indices: None,
            },
        }
    }

    pub fn play_track(&mut self, idx: usize) {
        self.playback.current_track = Some(idx);
    }

    pub fn next_track_idx(&self) -> usize {
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

    pub fn handle_normal_mode(
        &mut self,
        key_code: KeyCode,
        player: &rodio::Player,
        config: &AppConfig,
    ) {
        match key_code {
            KeyCode::Esc => {
                if self.input_state.filtered_indices.is_some() {
                    self.input_state.search_query.clear();
                    self.input_state.filtered_indices = None;
                }
            }
            KeyCode::Char('l') => {
                let new_pos = player.get_pos() + Duration::from_secs(config.skip_interval_secs);
                let _ = player.try_seek(new_pos);
            }
            KeyCode::Char('h') => {
                let new_pos = player
                    .get_pos()
                    .checked_sub(Duration::from_secs(config.skip_interval_secs))
                    .unwrap_or(Duration::ZERO);
                let _ = player.try_seek(new_pos);
            }
            KeyCode::Char('j') | KeyCode::Down => self.table_state.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.table_state.select_previous(),
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.playback.volume_level = (self.playback.volume_level + 0.05).min(1.0);
                player.set_volume(self.playback.volume_level);
            }
            KeyCode::Char('-') => {
                self.playback.volume_level = (self.playback.volume_level - 0.05).max(0.0);
                player.set_volume(self.playback.volume_level);
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                let next_idx = self.next_track_idx();

                if let Some(source) = get_track_source(next_idx, self) {
                    player.stop();
                    player.append(source);
                    self.play_track(next_idx);
                }
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                let prev_idx = self.prev_track_idx();

                if let Some(source) = get_track_source(prev_idx, self) {
                    player.stop();
                    player.append(source);
                    self.play_track(prev_idx);
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
                self.playback.is_random_shuffle = !self.playback.is_random_shuffle;
            }
            KeyCode::Enter => {
                if let Some(selected) = self.table_state.selected() {
                    let real_idx = self
                        .input_state
                        .filtered_indices
                        .as_ref()
                        .map(|indices| indices[selected])
                        .unwrap_or(selected);

                    if let Some(source) = get_track_source(real_idx, self) {
                        player.stop();
                        player.append(source);
                        self.playback.current_track = Some(real_idx);
                    }
                }
            }
            KeyCode::Char('/') => {
                self.input_state.mode = InputMode::Search;
            }
            _ => {}
        }
    }
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            current_track: None,
            started_at: Some(Instant::now()),
            paused: false,
            position: Duration::new(0, 0),
            volume_level: 1.0,
            is_random_shuffle: false,
        }
    }
}
