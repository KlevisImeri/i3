#!/bin/bash

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)

sudo dnf install -y \
    rust \
    cargo \
    i3 \
    i3status \
    i3lock \
    xss-lock \
    xrandr \
    network-manager-applet \
    udiskie \
    feh \
    dex-autostart \
    dmenu \
    alacritty \
    flameshot \
    pulseaudio-utils

cargo build --release --locked --manifest-path "$SCRIPT_DIR/display/Cargo.toml"
