# Configuration

spotimon stores its configuration at `~/.config/spotimon/config.toml`. If the file doesn't exist, it will be created with default values on first run.

## Example

```toml
[device]
volume = 50

music_folder = "/home/user/Music"
skip_interval_secs = 5
notify_message_live_seconds=3
```

## Options

### `music_folder`

|                 |                                                                                                                                                                                                                     |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Type**        | String (path)                                                                                                                                                                                                       |
| **Default**     | `~/Music`                                                                                                                                                                                                           |
| **Description** | Path to the directory containing your music files. spotimon recursively scans this directory for supported audio files (`.mp3`, `.flac`, `.ogg`). If the directory doesn't exist, it will be created automatically. |

### `skip_interval_secs`

|                 |                                                 |
| --------------- | ----------------------------------------------- |
| **Type**        | Integer                                         |
| **Default**     | `5`                                             |
| **Description** | How much volume up / down when you click + / -. |

### `notify_message_live_seconds`

|                 |                                                                       |
| --------------- | --------------------------------------------------------------------- |
| **Type**        | Integer                                                               |
| **Default**     | `3`                                                                   |
| **Description** | How much toast messages (e.g 'deleted from yokko!') lives in seconds. |

### `[device]`

#### `device.volume`

|                 |                                                                                                                        |
| --------------- | ---------------------------------------------------------------------------------------------------------------------- |
| **Type**        | Integer                                                                                                                |
| **Default**     | `50`                                                                                                                   |
| **Range**       | `0` — `100`                                                                                                            |
| **Description** | Default volume level on startup. Maps to a `0.0`—`1.0` float internally. Can be adjusted at runtime with `+`/`-` keys. |

## Supported Formats

spotimon supports the following audio formats through the symphonia decoder:

| Format     | Extensions |
| ---------- | ---------- |
| MP3        | `.mp3`     |
| FLAC       | `.flac`    |
| Ogg Vorbis | `.ogg`     |

## File Locations

| File            | Path                             |
| --------------- | -------------------------------- |
| Config          | `~/.config/spotimon/config.toml` |
| Music (default) | `~/Music`                        |
