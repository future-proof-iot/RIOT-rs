# Laze

Ariel OS makes use of the Laze build system to run cargo / RIOT's make with the
correct parameters for a specific board / application.

Laze has a builtin `laze build -b <board>` command, which in Ariel OS, maps to
`cargo build`.

For other tasks like flashing and debugging, Ariel OS uses Laze "tasks".
laze commands are applied to the application(s) within the subfolder laze is called.
For example, when called in `examples/hello-world`, `laze build -b nrf52840dk`
would build the hello-world example for nrf52840dk.

laze tasks currently have the syntax `laze build -b <board> [other options] <task-name>`.
E.g., to flash the bottles example, the command would be (when in `examples/bottles`):

    laze build -b nrf52840dk flash

Laze allows enabling/disabling features using "modules", which can be selected
or disabled using `--select <module>` or `--disable <module>>`.

Laze also allows to override global variables using e.g., `-DFOO=BAR`.

Note: all tasks and build need to be called with the same set of arguments
(`--select`, `--disable`, `-D...`).
A `laze build -DFOO=1 flash` followed by `laze build -DFOO=other debug` might not
work.

## Laze tasks

2023-10-14 Note: this is probably outdated

- `flash` -> compiles (if needed) and flashes an application
- `flash-riotboot` -> same as flash, but flashes to riotboot slot 0 (first slot)
  This needs the "riotboot" feature to be enabled. e.g.,

        laze task -b nrf52840dk --select riotboot flash-riotboot

  This does not flash the bootloader itself ATM, please use RIOT for that.

- `debug` -> starts a gdb debug session for the selected application.
  The application needs to be flashed using the `flash` or `flash-riotboot` tasks
  before starting `debug`.

- `reset` -> reboots the target
- `term` -> starts a serial terminal to the target's default port (currently
  hard-coded to use picocom on /dev/ttyACM0)

## Laze modules

This is an non-exhaustive list of modules that can be used.
E.g., to start a debug session with all semihosting and panic output enabled,
run

    laze build -b <board> flash
    laze build -b <board> debug

- `no-semihosting`: turn off Ariel OS debug output
  Note: semihosting output only gets printed in a debug session.
  Furthermore, it freezes the board on first output when not run in a
  debug session.
- `silent-panic`: don't print Ariel OS panics. Unless used elsewhere, this saves
  ~15k code on Cortex-M
- `riotboot`: configures the build to link to the first riotboot slot.
