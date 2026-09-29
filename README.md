# Patch Builder

Desktop tool that turns a game client folder (e.g. Lineage 2) into an update tree that launchers download from any HTTP server or CDN.

## How it works

1. **Scan**: hashes every file in the Source folder with BLAKE3 and compares it with the `manifest.json` already in the Update tree. Runs when the app opens, when a folder changes, and on Rescan.
2. **Build**: zips only new and changed files, one `.zip` per file, deletes archives of removed files, then writes the manifest last.

```
Source/                      Update tree/
  system/Fonts.utx     ->      system/Fonts.utx.zip
  Maps/23_21.unr       ->      Maps/23_21.unr.zip
                               manifest.json
```

```json
{
  "version": 2,
  "hash_algorithm": "blake3",
  "files": [{ "path": "system/Fonts.utx", "size": 29779558, "hash": "a3f1e7…" }]
}
```

`size` and `hash` describe the uncompressed file, so a launcher compares them with the player's install and downloads `path` + `.zip` for anything that differs. `version` only grows when the file list changes.

## Tech

- Rust 2024
- [Slint](https://slint.dev) for the UI (winit + femtovg), Inter and JetBrains Mono embedded
- BLAKE3 and rayon for parallel hashing, `zip` with zlib-rs for Deflate
- serde/serde_json for the manifest and saved settings, rfd for native dialogs

## Usage

```sh
make build              # optimized binary: target/release/patch-builder.exe
cargo run               # debug build
cargo run -- --preview  # UI with sample data, no folders needed
cargo test
```

The chosen folders are saved in `%APPDATA%\Patch Builder\settings.json`.

## Docs

- `CONTEXT.md`: domain glossary (Source, Archive, Manifest, Update tree, Scan, Plan, Build)
- `docs/adr/`: design decisions
