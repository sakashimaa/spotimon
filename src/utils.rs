use std::{fs::File, io::BufReader};

use rodio::Decoder;

use crate::state::App;

pub fn get_track_source(idx: usize, app_state: &App) -> Option<Decoder<BufReader<File>>> {
    if let Some(track) = app_state.library.tracks.get(idx)
        && let Ok(file) = File::open(&track.path)
        && let Ok(source) = Decoder::try_from(file)
    {
        return Some(source);
    }

    None
}
