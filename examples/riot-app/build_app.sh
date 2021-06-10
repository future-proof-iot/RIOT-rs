#!/bin/sh

if [ -n "$1" ]; then
    ACTION="${1}"
    shift
else
    ACTION="build"
fi

cargo $ACTION --features "boards/$BOARD riot-build/riot-rs-core" --target thumbv7em-none-eabi "$@"
