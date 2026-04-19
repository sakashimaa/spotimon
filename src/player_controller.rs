use std::{
    fs::{self, File},
    io::Write,
    sync::mpsc,
    time::Duration,
};

use rodio::{Player, play};
use souvlaki::{MediaControls, MediaMetadata};

use crate::{
    lyrics,
    state::{Action, App, Playlist, PlaylistManager, SortOrder},
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

            if !app.playback.queue.is_empty() {
                app.playback.queue.remove(0);
            }

            execute(Action::Play(next_idx), player, app, lyrics_tx, controls)
        }
        Action::PrevTrack => {
            let prev_idx = app.prev_track_idx();
            execute(Action::Play(prev_idx), player, app, lyrics_tx, controls)
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
        Action::ToggleMute => {
            if app.playback.volume_level <= 0.0 {
                app.playback.volume_level = app.playback.prev_volume;
            } else {
                app.playback.prev_volume = app.playback.volume_level;
                app.playback.volume_level = 0.0;
            }

            player.set_volume(app.playback.volume_level);
            false
        }
        Action::ToggleRepeat => {
            app.playback.repeat = !app.playback.repeat;
            false
        }
        Action::AddToQueue(idx) => {
            app.playback.queue.push(idx);
            false
        }
        Action::CreatePlaylist(name) => {
            if let Ok(r) = fs::exists(&app.playlist_manager.path)
                && !r
                && let Err(_) = File::create(&app.playlist_manager.path)
            {
                return false;
            }

            let _ = fs::read_to_string(&app.playlist_manager.path)
                .map(|s| app.playlist_manager.playlists = toml::from_str(&s).unwrap_or_default())
                .and_then(|_| {
                    app.playlist_manager
                        .playlists
                        .insert(name, Playlist { tracks: vec![] });
                    fs::write(
                        &app.playlist_manager.path,
                        toml::to_string(&app.playlist_manager.playlists)
                            .unwrap_or_default()
                            .as_bytes(),
                    )
                });

            false
        }
        Action::None => false,
        _ => false,
    }
}
