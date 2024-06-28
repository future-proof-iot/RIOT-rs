# defmt

RIOT-rs supports [defmt] on all platforms.
To enable it, enable the laze module `defmt`, either on the laze command line or
in a laze file. Don't forget to set the `DEFMT_LOG` variable, it defaults to `error`.
See the [defmt documentation] for general info on `defmt`.

Example:

```shell
# DEFMT_LOG=info laze build -C examples/hello-world-async --builders nrf52840dk --select defmt run
```

Then within Rust code, import `riot_rs::debug::log` items, then use `defmt` log
macros [as usual][defmt-macros]:

```rust
use riot_rs::debug::log::*;

#[riot_rs::task(autostart)]
async fn main() {
    info!("Hello!");
}
```

If the `defmt` laze module is not selected, all log statements become no-ops.

Note: On Cortex-M devices, the order of `riot_rs::debug::println!()` output and
      `defmt` log output is not deterministic.

[defmt]: https://github.com/knurling-rs/defmt
[defmt documentation]: https://defmt.ferrous-systems.com/
[defmt-macros]: https://defmt.ferrous-systems.com/macros
