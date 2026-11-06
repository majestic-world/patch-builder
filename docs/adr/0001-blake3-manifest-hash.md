# BLAKE3 as the Manifest hash

The Manifest hash is a contract every Launcher implements, and Launchers hash the whole installation on players' machines, so hashing speed matters on both ends. We chose BLAKE3 over SHA-256 and MD5 (common in existing Lineage 2 launchers) because it is the fastest option and parallelises within a single file; there is no legacy Launcher to stay compatible with. Changing it later means updating every deployed Launcher.

## Updated: selectable hashes

BLAKE3 remains the default, but the publisher can also select CRC32 or both algorithms. Existing BLAKE3-only Manifests keep their format. A Launcher must understand the selected `hash_algorithm` and its fields; changing the selection publishes a new Manifest version even when the Source files are unchanged. CRC32 is useful for compatibility and accidental-corruption detection, not cryptographic integrity.
