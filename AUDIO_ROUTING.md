# Per-Window Audio Source Routing — Status & Findings

## Goal

Each vibe window should capture from a different audio source. Example: window-1 reacts to Spotify, window-7 reacts only to TTS speech.

## Architecture

- Each vibe window is a separate OS process (`vibe window-1`, `vibe window-7`)
- Vibe uses CPAL (Rust audio lib) with the **ALSA backend**, which goes through **pipewire-alsa**
- Per-window config in `~/dotfiles/private/vibe/output_configs/window-N.toml` has an `audio_target` field
- TTS audio is produced by `clone-say` which calls ffplay to play generated speech
- A persistent PipeWire null sink called `tts_sink` exists for TTS audio (`~/.config/pipewire/pipewire.conf.d/tts-sink.conf`)

## What Works

### Vibe capture routing via PIPEWIRE_NODE (current approach)

Setting `PIPEWIRE_NODE=tts_sink` before CPAL opens its stream makes pipewire-alsa route the capture to tts_sink's monitor. Confirmed working:

- Standalone CPAL test program with `PIPEWIRE_NODE=tts_sink` receives audio from tts_sink.monitor (peaks at 0.55)
- `wpctl status` confirms vibe window-7 connects to `TTS Audio:monitor_FL/FR`
- Code: `window.rs` sets `std::env::set_var("PIPEWIRE_NODE", target)` before `config.sample_processor()`, restores after

### What doesn't work for vibe capture

| Approach | Result |
|----------|--------|
| `pactl move-source-output` | Graph links show correct but ALSA data path doesn't actually reroute — zero audio received |
| `PIPEWIRE_NODE=tts_sink.monitor` (with .monitor suffix) | No effect |
| `PIPEWIRE_PROPS='{ node.target = ... }'` | No effect |
| `pactl set-default-source` swap | PipeWire re-routes ALL existing streams when default changes, breaking other windows |
| CPAL device enumeration | ALSA host only sees hardware devices, not PipeWire virtual sinks |

## The Remaining Problem: TTS Playback Routing

`clone-say` uses ffplay (SDL) to play TTS audio. It needs to output to `tts_sink` so window-7 can capture it. **None of the standard approaches work** because PipeWire's `module-stream-restore` overrides them:

| Approach | Result |
|----------|--------|
| `PULSE_SINK=tts_sink ffplay ...` | stream-restore sends ffplay to last-used sink (Audeze) |
| `PIPEWIRE_NODE=tts_sink ffplay ...` | Same — stream-restore overrides |
| `SDL_AUDIODRIVER=pipewire PIPEWIRE_NODE=tts_sink ffplay ...` | Same |
| `pactl move-sink-input <id> tts_sink` | **Works** — but ffplay is short-lived, timing is tricky |

The `pactl move-sink-input` approach is implemented in `clone-say` now but has a race: it polls for the sink-input by PID every 100ms for up to 2 seconds. If ffplay finishes before the move completes, TTS audio goes to the wrong sink. For short utterances this may fail.

## Current State of Code

### `vibe/src/window.rs`
- Sets `PIPEWIRE_NODE` env var before `config.sample_processor()` when `audio_target` is configured
- Restores previous `PIPEWIRE_NODE` value after stream opens
- The old `move_source_output_to()` function has been removed

### `vibe/src/output/config/mod.rs`
- `OutputConfig` has `audio_target: Option<String>` with `#[serde(default)]`

### `~/dotfiles/private/bin/clone-say`
- Has `move_to_tts_sink()` function that polls for ffplay's sink-input by PID and moves it
- Race condition: short TTS may play to wrong sink before move completes

### `~/.config/pipewire/pipewire.conf.d/tts-sink.conf`
- Creates persistent `tts_sink` null-audio-sink
- Loopback module was removed (caused feedback)

### Output configs
- `window-1.toml`: no `audio_target` (uses default source — Audeze monitor for Spotify)
- `window-7.toml`: `audio_target = "tts_sink"`

## Key Insight

The core unsolved problem is **PipeWire stream-restore**. It remembers per-application sink/source assignments and overrides env vars and explicit targeting. This affects both:

1. **ffplay output** → stream-restore routes "SDL Application" / media.role "game" to the last-used sink
2. **vibe capture** → stream-restore routes "PipeWire ALSA [vibe]" to the last-used source (this is why we needed `PIPEWIRE_NODE` set before stream creation, not `pactl move` after)

Possible directions to explore:
- Disable stream-restore for specific applications via PipeWire rules
- Use `pw-play --target tts_sink` instead of ffplay in clone-say (avoids SDL/stream-restore entirely)
- Set `stream.restore.target` PipeWire property to prevent restore override
- Replace ffplay with a PipeWire-native player that respects `--target`
