#!/bin/sh

if [ -n "$1" ]; then
    ACTION="${1}"
    shift
else
    ACTION="build"
fi

cargo $ACTION --features "boards/$BOARD" --target thumbv7em-none-eabi "$@"
