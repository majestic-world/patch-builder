# Patch Builder

Turns a game client directory into an Update tree that Launchers consume to keep players' installations (e.g. Lineage 2) up to date.

## Language

**Source**:
The input directory tree, laid out exactly as the game client is installed, holding only the files Launchers must keep identical on players' machines.
_Avoid_: input folder, game folder

**Source file**:
A file inside the Source, identified by its path relative to the Source root (e.g. `system/Fonts.utx`).
_Avoid_: input file, game file

**Archive**:
The compressed copy of exactly one Source file, stored at the same relative path with `.zip` appended (e.g. `system/Fonts.utx.zip`).
_Avoid_: zip, package, patch file

**Manifest**:
The JSON document listing every Source file of the current Source with its size and hash, both taken from the uncompressed Source file; the single entry point a Launcher reads.
_Avoid_: hash file, hash list, index

**Manifest version**:
A number in the Manifest that grows by one whenever its file list changes; a Launcher that already installed that version has nothing to download.
_Avoid_: build number, revision, patch level

**Update tree**:
The published output of a Build: the Manifest plus every Archive, mirroring the Source layout, served as static files from any HTTP server or CDN.
_Avoid_: patch, output folder

**Scan**:
Hashing every Source file and comparing it with the Update tree's current Manifest, without writing anything; it runs when the app opens, when a folder is chosen, and on Rescan.
_Avoid_: analysis, check, verification

**Plan**:
The result of the last Scan, kept in memory: each Source file's status and the Archives to remove; what the next Build publishes.
_Avoid_: diff, changeset, queue

**Build**:
Applying the Plan to the Update tree: re-zipping its New and Changed files, deleting the Archives of removed ones, and writing the Manifest.
_Avoid_: patch, release, generation

**Launcher**:
An external application that compares a player's installation against the Manifest and downloads the Archives of files that differ.
_Avoid_: updater, client
