<!--
Release Notes Template:

## <version> - <date iso8601> - <name>
<short description>

### Fixes
<fixed bugs>

### Added
<added features and commands>

### Removed
<removed features and commands>

### Known Bugs
- [#n](<link>): <description>

-->
# Changelog

## v1.0.0 - 2021-10-08 - Initial Release
The initial release of the Sunny Flowers Discord music bot.

### Added
Sunny now supports the following commands:
- `join`: joins your voice channel
- `leave`: leaves the voice channel
- `play`: adds a song to the queue
- `play_next`: adds a song to the front of the queue
- `pause`: pauses the playback of the current song
- `resume`: resumes the playback of the current song
- `skip`: skips to the next song in the queue
- `stop`: stops the current song and clears queue
- `shuffle`: shuffles the queue
- `swap`: swaps two numbers in the queue from position
- `remove_at`: removes a song from the queue at a specific index
- `now_playing`: shows an embed containg info of the currently playing song
- `queue`: displays the songs coming up in the queue
- `help`: shows available commands, their aliases, and usage.

### Known Bugs
- [#16](https://github.com/Druue/Sunny-Flowers/issues/16): Playlists behave buggy
- [#39](https://github.com/Druue/Sunny-Flowers/issues/39): Song metadata may not show correctly for raw music files
- [#54](https://github.com/Druue/Sunny-Flowers/issues/54): YouTube's age restricted videos don't play and fail silently
