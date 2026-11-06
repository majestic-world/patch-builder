# A Build rewrites its Update tree in place

The previous Build is whatever Manifest the chosen Update tree already holds; there is no separate "previous Update tree" input. Unchanged Archives stay untouched (their bytes and CDN cache entries survive), Archives of files that left the Source are deleted, and the Manifest is replaced last and atomically so a Launcher never sees it list an Archive that does not exist yet. The Manifest version grows when the file list or selected hash algorithms change, so a Build with nothing new leaves the Manifest as it was.
