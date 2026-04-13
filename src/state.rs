use std::time::{Duration, Instant};

use rand::RngExt;
use ratatui::widgets::TableState;

use crate::track_library::TrackLibrary;

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
