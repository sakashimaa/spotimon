use std::{sync::mpsc, time::Duration};

use rodio::Player;
use souvlaki::{MediaControls, MediaMetadata};

use crate::{
    lyrics,
    state::{Action, App, SortOrder},
    utils::get_track_source,
};

// returns true if should exit
pub fn execute(
    action: Action,
    player: &Player,
    app: &mut App,
    lyrics_tx: &mpsc::Sender<Option<String>>,
    controls: &mut MediaControls,
) -> bool {
    match action {
        Action::Quit => {
            if app.input_state.filtered_indices.is_some() {
                app.input_state.search_query.clear();
                app.input_state.filtered_indices = None;
            }

            false
        }
        Action::SeekForward(duration) => {
            let new_pos = player.get_pos() + duration;
            let _ = player.try_seek(new_pos);

            false
        }
        Action::SeekBackward(duration) => {
            let new_pos = player
                .get_pos()
                .checked_sub(duration)
                .unwrap_or(Duration::ZERO);
            let _ = player.try_seek(new_pos);

            false
        }
        Action::NavigateDown => {
            app.table_state.select_next();
            false
        }
        Action::NavigateUp => {
            app.table_state.select_previous();
            false
        }
        Action::SetVolume(step) => {
            player.set_volume(step);
            false
        }
        Action::NextTrack => {
            let next_idx = app.next_track_idx();

            if let Some(source) = get_track_source(next_idx, app) {
                player.stop();
                player.append(source);
                app.play_track(next_idx);
            }

            false
        }
        Action::PrevTrack => {
            let prev_idx = app.prev_track_idx();

            if let Some(source) = get_track_source(prev_idx, app) {
                player.stop();
                player.append(source);
                app.play_track(prev_idx);
            }

            false
        }
        Action::Pause => {
            if player.is_paused() {
                player.play();
            } else {
                player.pause();
            }

            false
        }
        Action::ToggleShuffle => {
            app.playback.is_random_shuffle = !app.playback.is_random_shuffle;

            false
        }
        Action::Play(idx) => {
            app.cover_protocol = app.library.tracks[idx]
                .cover
                .as_ref()
                .and_then(|bytes| image::load_from_memory(bytes).ok())
                .map(|img| app.picker.new_resize_protocol(img));

            if let Some(source) = get_track_source(idx, app) {
                if player.is_paused() {
                    player.play();
                }
                player.stop();
                player.append(source);
                app.playback.current_track = Some(idx);
            }

            let artist = app.library.tracks[idx].artist.clone();
            let title = app.library.tracks[idx].title.clone();
            let album = app.library.tracks[idx].album.clone();

            let _ = controls.set_metadata(MediaMetadata {
                title: Some(&title),
                artist: Some(&artist),
                album: Some(&album),
                ..Default::default()
            });

            let tx = lyrics_tx.clone();

            std::thread::spawn(move || {
                let result = lyrics::fetch(&artist, &title).and_then(|r| r.plain_lyrics);
                let _ = tx.send(result);
            });

            app.playback.lyrics = Some("Loading...".to_string());

            false
        }
        Action::ToggleInputMode(mode) => {
            app.input_state.mode = mode;

            false
        }
        Action::ToggleViewMode(mode) => {
            app.view_mode = mode;

            false
        }
        Action::Sort(field) => {
            if app.sort_state.field == field {
                app.sort_state.order = match app.sort_state.order {
                    SortOrder::Asc => SortOrder::Desc,
                    SortOrder::Desc => SortOrder::Asc,
                };
            } else {
                app.sort_state.field = field;
                app.sort_state.order = SortOrder::Asc;
            }

            app.apply_sort();
            false
        }
        Action::None => false,
        _ => false,
    }
}
