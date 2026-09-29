# The Update tree is static files; the Patch Builder serves no API

Launchers consume "the API" by fetching the Manifest and Archives over plain HTTP from any web server or CDN. The Patch Builder only produces the Update tree and runs no server, keeping it a single-purpose build tool and letting publishers host the output wherever they want.
