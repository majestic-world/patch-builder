# Patch Builder

## System Overview

Desktop tool that turns a game client folder (for example, Lineage 2) into an Update tree that Launchers download from any HTTP server or CDN.

1. **Scan** hashes every file in the Source with the selected algorithms and compares it with the `manifest.json` already in the Update tree. It runs when the app opens, when a folder or hash selection changes, and on Rescan.
2. **Build** applies the in-memory Plan: it zips only New and Changed files, deletes Archives for removed files, and writes the Manifest last.

```text
Source/                      Update tree/
  system/Fonts.utx     ->      system/Fonts.utx.zip
  Maps/23_21.unr       ->      Maps/23_21.unr.zip
                               manifest.json
```

The Manifest is the Launcher's entry point:

```json
{
  "version": 2,
  "hash_algorithm": "blake3+crc32",
  "files": [
    {
      "path": "system/Fonts.utx",
      "size": 29779558,
      "hash": "<BLAKE3 hex>",
      "crc32": "<CRC32 hex>"
    }
  ]
}
```

A Launcher compares these entries with the player's installation and downloads `path + ".zip"` for each differing file. CRC32 detects accidental corruption but is not a cryptographic hash.

## Local Interface & Invariants

- The root Cargo package is the Rust 2024 binary `patch-builder`; its release profile uses optimization level 3, disables cross-crate LTO, and keeps 16 code-generation units. [`Cargo.toml:package`, `Cargo.toml:profile.release`]
- `make build` is the default Make target. It runs `tools/build.ps1` with PowerShell 7 and publishes the optimized binary as `dist/Builder.exe`. [`Makefile:build`]
- The Cargo build script always compiles `ui/app.slint`. On Windows targets it renders `ui/icons/app-icon.svg` at eight fixed sizes, writes `$OUT_DIR/app.ico`, and embeds that resource in the executable. [`build.rs:main`, `build.rs:ICON_SIZES`, `build.rs:write_ico`]
- Release builds omit `target-cpu=native`; the binary must remain usable on publishers' machines other than the build host. [`Cargo.toml:profile.release`]

## Subdirectories & System Boundaries

- [Application core](./src/README.md) — Scan, Plan, Build, Manifest persistence, comparison tree, settings, and Slint session orchestration.

## Global Invariants

- BLAKE3 is the default. A publisher may select BLAKE3, CRC32, or both, but at least one remains selected. `hash_algorithm` is respectively `blake3`, `crc32`, or `blake3+crc32`. [`src/manifest.rs:HashAlgorithm`, `src/session.rs:Session::start`]
- With one algorithm, `hash` contains that algorithm's value. With both, `hash` contains BLAKE3 and `crc32` contains CRC-32/ISO-HDLC as eight lowercase hexadecimal digits. All hashes and `size` describe the uncompressed Source file. [`src/scan.rs:Hashes::selected`, `src/scan.rs:Hashers::finish`, `src/manifest.rs:Entry`]
- Manifest `version` starts at 1 and grows only when the file list or selected algorithms change. Unchanged Archives remain reusable. [`src/scan.rs:Plan::next_version`, `src/comparison.rs:FileStatus::Unchanged`]
- A Build applies the last Scan's Plan. It refuses to publish when the Update-tree Manifest changed after Scan or a file being archived no longer matches its scanned size and hashes. [`src/build.rs:run`, `src/build.rs:write_archive`]
- Archives are replaced from temporary files, and `manifest.json` is atomically replaced only after every Archive it lists exists. [`src/build.rs:write_archive`, `src/build.rs:run`, `src/manifest.rs:write`]
- A Launcher must support the selected Manifest format. Manifest and Archive paths preserve Source letter case and use `/` separators so the generated URLs match case-sensitive storage. [`src/scan.rs:relative_path`, `src/manifest.rs:archive_path`]

## Global Side Effects

- Scan recursively reads and hashes the Source, reads the current Manifest, and may decompress an existing Archive to verify BLAKE3; it does not write the Update tree. [`src/scan.rs:run`, `src/scan.rs:archive_matches_blake3`]
- Build writes one deflated Archive per New or Changed file, removes obsolete Archives and empty directories, and replaces the Manifest last. [`src/build.rs:run`, `src/build.rs:write_archive`, `src/build.rs:remove_archive`]
- The application stores the chosen folders, hash algorithms, and Light or Dark appearance under the platform configuration directory; on Windows this is `%APPDATA%\Patch Builder\settings.json`. [`src/settings.rs:file`, `src/settings.rs:save`]
- The desktop session opens native folder and error dialogs and opens the Update tree in the platform file manager after a successful Build. [`src/session.rs:pick_folder`, `src/session.rs:show_error`, `src/session.rs:open_folder`]

## Technology

- Rust 2024
- [Slint](https://slint.dev) for the UI (winit + femtovg), with embedded Inter and JetBrains Mono fonts
- BLAKE3 and CRC32 for hashing, Rayon for parallel work, and `zip` with zlib-rs for Deflate
- serde and serde_json for the Manifest and saved settings; rfd for native dialogs

## Usage

```sh
make build              # optimized binary: dist/Builder.exe (needs PowerShell 7)
cargo run               # debug build
cargo run -- --preview  # UI with sample data, no folders needed
```

## Documentation

- [`GLOSSARY.md`](./GLOSSARY.md) defines Source, Archive, Manifest, Update tree, Scan, Plan, Build, and Launcher.
- [`docs/adr/`](./docs/adr/) records architectural decisions.

## System Verification

- Run: `cargo test`
