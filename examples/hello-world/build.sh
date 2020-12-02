#!/bin/sh

if [ -n "$1" ]; then
    ACTION="${1}"
    shift
else
    ACTION="build"
fi

cargo $ACTION --features "riot-rs-boards/$BOARD" --target thumbv7em-none-eabi "$@"
