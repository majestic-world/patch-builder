# A Build applies the last Scan instead of re-reading the Source

Hashing a whole client takes long enough that the app Scans once (on open, on folder change, on Rescan) and keeps the resulting Plan in memory; Build only zips the files the Plan marks New or Changed. Edits made after the Scan are invisible until the user Rescans. To keep the Manifest truthful anyway, a Build refuses to run if the Update tree's Manifest changed since the Scan, and it hashes every byte it zips, stopping before the Manifest is written if a file no longer matches its scanned hash.
