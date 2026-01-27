#!/bin/bash

if [ "$XDG_SESSION_TYPE" == "wayland" ]; then
    IS_WAYLAND=true
else
    IS_WAYLAND=false
fi

INTERNAL_MONITOR="eDP-1"

if [ "$IS_WAYLAND" = true ]; then
    EXTERNAL_MONITOR=$(swaymsg -t get_outputs | grep '"name":' | grep -v "$INTERNAL_MONITOR" | cut -d '"' -f 4 | head -n 1)
    if [ ! -z "$EXTERNAL_MONITOR" ]; then
        echo "Sway: External monitor $EXTERNAL_MONITOR detected."
        swaymsg output "$INTERNAL_MONITOR" disable
        swaymsg output "$EXTERNAL_MONITOR" enable
    else
        echo "Sway: No external monitor. Enabling internal."
        swaymsg output "$INTERNAL_MONITOR" enable
    fi
else
  EXTERNAL_MONITORS=("HDMI-1-1" "HDMI-1-2" "HDMI-1-4" "HDMI-1" "HDMI-2", "HDMI-A-2")
  EXTERNAL_MONITOR_FOUND=false

  for MONITOR in "${EXTERNAL_MONITORS[@]}"; do
      STATUS=$(xrandr | grep "$MONITOR" | awk '{print $2}')
      if [[ "$STATUS" == "connected" ]]; then
          echo "External monitor $MONITOR detected. Setting it as primary and turning off internal monitor."
          xrandr --output "$MONITOR" --primary --auto --output "$INTERNAL_MONITOR" --off
          EXTERNAL_MONITOR_FOUND=true
          break 
      fi
  done

  if ! $EXTERNAL_MONITOR_FOUND; then
      echo "No external monitors detected. Setting $INTERNAL_MONITOR as primary."
      TURNOFF_COMMAND="xrandr --output \"$INTERNAL_MONITOR\" --primary --auto"
      for MONITOR in "${EXTERNAL_MONITORS[@]}"; do
          TURNOFF_COMMAND+=" --output \"$MONITOR\" --off"
      done
      eval "$TURNOFF_COMMAND"
  fi
fi
