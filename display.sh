#!/bin/bash

# Define monitor names
EXTERNAL_MONITOR_1="HDMI-1-1"
EXTERNAL_MONITOR_2="HDMI-1-4"
INTERNAL_MONITOR="eDP-1"

# Get the connection status of the external monitors
STATUS_MONITOR_1=$(xrandr | grep "$EXTERNAL_MONITOR_1" | awk '{print $2}')
STATUS_MONITOR_2=$(xrandr | grep "$EXTERNAL_MONITOR_2" | awk '{print $2}')

# Determine which monitor is connected and set it as primary
if [[ "$STATUS_MONITOR_1" == "connected" ]]; then
    echo "External monitor $EXTERNAL_MONITOR_1 detected. Setting it as primary."
    xrandr --output "$EXTERNAL_MONITOR_1" --primary --auto --output "$INTERNAL_MONITOR" --off
elif [[ "$STATUS_MONITOR_2" == "connected" ]]; then
    echo "External monitor $EXTERNAL_MONITOR_2 detected. Setting it as primary."
    xrandr --output "$EXTERNAL_MONITOR_2" --primary --auto --output "$INTERNAL_MONITOR" --off
else
    echo "No external monitors detected. Setting $INTERNAL_MONITOR as primary."
    xrandr --output "$INTERNAL_MONITOR" --primary --auto --output "$EXTERNAL_MONITOR_1" --off --output "$EXTERNAL_MONITOR_2" --off
fi

