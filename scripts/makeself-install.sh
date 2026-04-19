#!/bin/sh
# Anodizer installer — used by makeself self-extracting archive.
# Copies the anodizer binary to PREFIX/bin (default: /usr/local).
set -e

PREFIX="${PREFIX:-/usr/local}"
BINDIR="${PREFIX}/bin"

if [ ! -d "$BINDIR" ]; then
    mkdir -p "$BINDIR"
fi

if [ -f anodizer ]; then
    install -m 0755 anodizer "$BINDIR/anodizer"
    echo "Installed anodizer to $BINDIR/anodizer"
else
    echo "Error: anodizer binary not found in archive" >&2
    exit 1
fi
