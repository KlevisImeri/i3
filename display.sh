#!/bin/bash

EXTERNAL_MONITORS=("HDMI-1-1" "HDMI-1-2" "HDMI-1-4" "HDMI-1" "HDMI-2")
INTERNAL_MONITOR="eDP-1"

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
