# Patch Builder

Desktop tool that turns a game client folder (e.g. Lineage 2) into an update tree that launchers download from any HTTP server or CDN.

## How it works

1. **Scan**: hashes every file in the Source folder with the selected algorithms and compares it with the `manifest.json` already in the Update tree. Runs when the app opens, when a folder or hash selection changes, and on Rescan.
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
  "hash_algorithm": "blake3+crc32",
  "files": [{ "path": "system/Fonts.utx", "size": 29779558, "hash": "<BLAKE3 hex>", "crc32": "<CRC32 hex>" }]
}
```

Choose BLAKE3, CRC32, or both before building. BLAKE3 is the default and preserves the existing manifest format. With a single selection, `hash_algorithm` is `blake3` or `crc32` and `hash` contains that algorithm's value. With both selected, `hash_algorithm` is `blake3+crc32`, `hash` contains BLAKE3, and `crc32` contains CRC-32/ISO-HDLC as eight lowercase hexadecimal digits. All hashes and `size` describe the uncompressed file.

A launcher must support the selected manifest format; it compares the hashes with the player's install and downloads `path` + `.zip` for anything that differs. CRC32 detects accidental corruption but is not a cryptographic hash. `version` grows when the file list or selected algorithms change; unchanged Archives can still be reused.

## Tech

- Rust 2024
- [Slint](https://slint.dev) for the UI (winit + femtovg), Inter and JetBrains Mono embedded
- BLAKE3 and CRC32 for hashing, rayon for parallel work, `zip` with zlib-rs for Deflate
- serde/serde_json for the manifest and saved settings, rfd for native dialogs

## Usage

```sh
make build              # optimized binary: dist/Builder.exe (needs PowerShell 7)
cargo run               # debug build
cargo run -- --preview  # UI with sample data, no folders needed
cargo test
```

The chosen folders and hash algorithms are saved in `%APPDATA%\Patch Builder\settings.json`.

## Docs

- `CONTEXT.md`: domain glossary (Source, Archive, Manifest, Update tree, Scan, Plan, Build)
- `docs/adr/`: design decisions
