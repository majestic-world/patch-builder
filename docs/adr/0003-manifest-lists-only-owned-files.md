# The Manifest lists only files Launchers own; Launchers leave everything else alone

The Manifest is the complete list of the current Source and carries no per-file policy (overwrite, create-if-missing, delete). Launchers ignore local files absent from the Manifest, so a file dropped from the Source stays on players' machines, and player-edited files such as `system/option.ini` are kept out of the Source by the publisher. We rejected per-file policies to keep the Manifest a plain snapshot.
