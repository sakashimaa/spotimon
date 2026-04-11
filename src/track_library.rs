use std::path::PathBuf;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::prelude::ItemKey;
use walkdir::WalkDir;

pub struct TrackLibrary {
    pub tracks: Vec<Track>,
}

pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: std::time::Duration,
    #[allow(unused)]
    pub path: PathBuf,
}

impl TrackLibrary {
    pub fn new(music_folder: &PathBuf) -> Self {
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
