#!/bin/bash

trap "exit" INT TERM
trap "kill 0" EXIT

set -x
openocd -f "board/nordic_nrf52_dk.cfg" -c 'init' -c 'targets' &

arm-none-eabi-gdb -q -x ${SCRIPTS}/openocd.gdb "$@"
