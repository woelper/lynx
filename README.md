# lynx
## A _very_ minimal audio player

This is a for-fun minimal audio player. It's written as a challenge to replicate something that works like xmms or winamp in Rust.

![screenshot](screenshot.png)

Features:
- Playlist with drag and drop
- Bookmarks within individual songs -  helpful to bookmark multiple individual audiobooks
- Favourite list
- Play count is recorded

Formats supported:
- wav
- flac
- mp3
- ogg

TODO:
- [ ] Keyboard shortcuts
- [ ] Themes
- [ ] Preload next song 
- [X] Recursively add dropped folders
- [x] Bookmark support for individual files (For multiple audiobooks)
- [X] Favorites
- [X] Play count
- [X] Auto-build for mac,win and linux
- [X] Drag and drop files into window
- [x] Scrub through songs

ISSUES:
Since the underlying sound library Kira does not stream sounds (yet), there will a small delay when you skip through sounds as they need to be fully loaded into memory. For that penalty, you get instant scrubbing and no-latency jumping between bookmarks.
