## laze

RIOT-rs makes use of the laze build system to run cargo / RIOT's make with the
correct parameters for a specific board / application.

laze has a builtin `laze build -b <board>` command, which in RIOT-rs, maps to
`cargo build`.

For other tasks like flashing and debugging, RIOT-rs uses laze "tasks".
laze commands are applied to the application(s) within the subfolder laze is called.
For example, when called in `examples/hello-world`, `laze build -b nrf52840dk`
would build the hello-world example for nrf52840dk.

laze tasks currently have the syntax `laze task -b <board> [other options] <task-name>`.
E.g., to flash the bottles example, the command would be (when in `examples/bottles`):

    laze task -b nrf52840dk flash

laze allows enabling/disabling features using "modules", which can be selected
or disabled using `--select <module>` or `--disable <module>>`.

laze also allows to override global variables using e.g.,  `-DFOO=BAR`.

Note: all tasks and build need to be called with the same set of arguments
(`--select`, `--disable`, `-D...`).
A `laze task -DFOO=1 flash` followed by `laze task -DFOO=other debug` might not
work.

### laze tasks

- `flash` -> compiles (if needed) and flashes an application
- `flash-riotboot` -> same as flash, but flashes to riotboot slot 0 (first slot)
   This needs the "riotboot" feature to be enabled. e.g.,

    laze task -b nrf52840dk --select riotboot flash-riotboot

   This does not flash the bootloader itself ATM, please use RIOT for that.

- `debug` -> starts a gdb debug session for the selected application.
  The application needs to be flashed using the `flash` or `flash-riotboot` tasks
  before starting `debug`.

### laze modules

- `release`: used by default. Selects `no-semihosting` and `silent-panic`
- `no-semihosting`: turn off RIOT-rs debug output
- `silent-panic`: don't print RIOT-rs panics. Unless used elsewhere, this saves
   ~15k code on Cortex-M
- `riotboot`: configures the build to link to the first riotboot slot.
