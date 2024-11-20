# defmt

Ariel OS supports [defmt] on all platforms. It is enabled by default.

See the [defmt documentation] for general info on `defmt`.

In Ariel OS, the log level defaults to `info`. It can be configured using the
laze variable `LOG`.

Example:

```shell
# laze build -C examples/log --builders nrf52840dk -DLOG=warn run
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
