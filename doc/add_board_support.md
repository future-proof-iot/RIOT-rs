# How to add a board

This assumes the board is already supported by RIOT (C).
Currently, each board needs special support in RIOT-rs, as not RIOT's bringup and
linking is uses, but Rust stuff.

## nrf52840 based

This is fairly straight forward. Currently, RIOT-rs doesn't do anything with
peripherals itself (only through the riot-wrappers), so all nrf52840 based
should work with the same code, apart from flashing.

1. copy `src/riot-rs-boards/nrf52840dk` to `src/riot-rs-boards/<board>`
1. replace all nrf52840dk in string literals with the new board name
1. add the new board to the features at src/riot-rs-boards/Cargo.toml, e.g.,

        new_board = { optional=true, path="new_board" }

1. add an entry in src/riot-rs-boards/src/lib.rs. replace underlines in the
   `pub use` but not in the feature name, e.g., if the board and feature
   `nrf52840-mdk` -> `pub use nrf52840_mdk as board;`.

1. in laze-project.yml, copy the "nrf52840dk" builder entry (and fix the name)

1. try it: `laze -Cexamples/bottles build -b <new-board>`

Assuming the board can be flashed with openocd/jlink, the flash task should work
as is. Otherwise, the flashing method needs to be added/fixed.
