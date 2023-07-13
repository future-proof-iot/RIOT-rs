#!/bin/bash

trap "exit" INT TERM
trap "kill 0" EXIT

set -x
openocd $OPENOCD_ARGS -c 'init' -c 'targets' -c "adapter speed 5000" &

arm-none-eabi-gdb -q -x ${SCRIPTS}/openocd.gdb "$@"
