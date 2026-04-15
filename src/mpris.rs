use std::{cell::Cell, sync::mpsc, time::Instant};

use souvlaki::{MediaControlEvent, MediaControls, PlatformConfig};

use crate::state::Action;

pub fn create_controls() -> Result<MediaControls, Box<dyn std::error::Error>> {
    let config = PlatformConfig {
        dbus_name: "spotimon",
        display_name: "Spotimon",
        hwnd: None,
    };

    let controls = MediaControls::new(config)?;
    Ok(controls)
}

pub fn attach_handler(controls: &mut MediaControls, tx: mpsc::Sender<Action>) {
    let last_event = Cell::new(Instant::now());

    controls
        .attach(move |event: MediaControlEvent| {
            // игнорируй события чаще чем раз в 200ms
            if last_event.get().elapsed().as_millis() < 200 {
                return;
            }
            last_event.set(Instant::now());

            let action = match event {
                MediaControlEvent::Play | MediaControlEvent::Pause | MediaControlEvent::Toggle => {
                    Action::Pause
                } // твой Pause уже делает toggle
                MediaControlEvent::Next => Action::NextTrack,
                MediaControlEvent::Previous => Action::PrevTrack,
                _ => Action::None,
            };
            let _ = tx.send(action);
        })
        .ok();
}
