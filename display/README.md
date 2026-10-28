# i3-display

A Rust display switcher for i3 and Sway.

- Prefers the first connected external monitor; otherwise uses the laptop screen.
- Disables all other outputs, including unplugged outputs still active in X11.
- Uses `xrandr` for i3/X11 and `swaymsg` for Sway.

i3 runs `~/.config/i3/display/target/release/i3-display` directly on startup and
restart. No shell wrapper is needed.
