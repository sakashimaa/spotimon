# Roadmap

## Quick wins
- [x] Update cheatsheet - missing playlist keybinds (t, c, :, d)
- [x] Context-aware top bar - shows "Track library" even in playlists/queue view. Switch label based on view_mode (e.g. "Playlists", "Queue", "chill (5 tracks)")

## Playlist management
- [ ] Delete entire playlist - 'D' in ViewMode::Playlists, Action::DeletePlaylist(name), playlists.remove(&name) + save
- [ ] Rename playlist

## Queue
- [ ] Remove individual items from queue
- [ ] Reorder queue items

## UX
- [ ] Confirmation prompt for destructive actions (delete playlist)
- [ ] Fuzzy search (current is substring match)
- [ ] Resume playback position on restart
