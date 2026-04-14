use serde::Deserialize;

use crate::track_library::Track;

#[derive(Deserialize)]
pub struct LrcLibResponse {
    #[serde(rename = "plainLyrics")]
    pub plain_lyrics: Option<String>,

    #[serde(rename = "syncedLyrics")]
    pub synced_lyrics: Option<String>,
}

pub fn fetch(artist: &str, title: &str) -> Option<LrcLibResponse> {
    let url = format!(
        "https://lrclib.net/api/get?artist_name={}&track_name={}",
        urlencoding::encode(artist),
        urlencoding::encode(title)
    );
    ureq::get(&url).call().ok()?.body_mut().read_json().ok()
}
