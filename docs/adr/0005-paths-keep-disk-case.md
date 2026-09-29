# Paths keep their on-disk letter case; case-only collisions fail the Build

Manifest paths and Archive paths use each Source file's name exactly as it is on disk, so the URL a Launcher builds from the Manifest always matches the Archive stored on a case-sensitive CDN. We rejected lowercasing every path because it silently renames files players see. Two Source files whose paths differ only in case (possible on Linux, impossible in a Windows install) fail the Build instead of producing an Update tree Launchers cannot apply.
