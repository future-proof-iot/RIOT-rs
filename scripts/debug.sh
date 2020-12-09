#!/bin/bash

trap "exit" INT TERM
trap "kill 0" EXIT

set -x
openocd $OPENOCD_ARGS -c 'init' -c 'targets' &

arm-none-eabi-gdb -q -x ${SCRIPTS}/openocd.gdb "$@"
