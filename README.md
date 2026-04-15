# spotimon

A minimalist TUI music player for local audio files, built in Rust with [ratatui](https://github.com/ratatui/ratatui) and [rodio](https://github.com/RustAudio/rodio).

## Features

- Browse and play local music library (MP3, FLAC, OGG)
- Automatic metadata parsing (title, artist, album, duration)
- Track progress bar with timing display
- Volume control
- Next/previous track navigation
- Shuffle mode
- Search/filter tracks by title, artist, or album
- Auto-advance to next track with queue looping
- Seek forward/backward
- Vim-style keybindings
- Watch lyrics

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/sakashimaa/spotimon/main/install.sh | sh
```

### Dependencies

On Linux (ALSA):

```bash
# Debian/Ubuntu
sudo apt install libasound2-dev

# Arch
sudo pacman -S alsa-lib

# Fedora
sudo dnf install alsa-lib-devel
```

## Usage

```bash
spotimon
```

On first run, spotimon creates a config file at `~/.config/spotimon/config.toml` with default settings. Place your music files in `~/Music` or configure a custom path.

## Keybindings

| Key       | Action                     |
| --------- | -------------------------- |
| `j` / `↓` | Navigate down              |
| `k` / `↑` | Navigate up                |
| `Enter`   | Play selected track        |
| `Space`   | Pause / Resume             |
| `n`       | Next track                 |
| `p`       | Previous track             |
| `h` / `←` | Seek backward 5s           |
| `l` / `→` | Seek forward 5s            |
| `+` / `=` | Volume up                  |
| `-`       | Volume down                |
| `s`       | Toggle shuffle             |
| `/`       | Search mode                |
| `Esc`     | Exit search / Clear filter |
| `q`       | Quit                       |
| `L`       | Lyrics of current track    |

## Configuration

See [docs/config.md](docs/config.md) for details.

## Tech Stack

- **TUI**: ratatui + crossterm
- **Audio playback**: rodio (symphonia backend)
- **Metadata**: lofty
- **Config**: serde + toml
- **Directory scanning**: walkdir

## License

MIT
