# RIOT-rs startup

This summarizes the startup sequence of RIOT-rs when building RIOT applications.

1. cortex-m-rt asm.S `bl main()` -> calls [riot-rs-rt main()](../src/riot-rs-rt/src/lib.rs)
2. riot-rs-rt calls `riot_rs_rt_startup()` from [riot-build](../src/riot-build/src/lib.rs)
3. riot-build `riot_rs_rt_startup()` calls [riot-rs-boards `board::init()`](../src/riot-rs-boards/src/lib.rs)
4. riot-rs-boards calls board/cpu specific startup code, then returns to `riot_rs_rt_startup()`
5. riot-build `riot_rs_rt_startup()` calls RIOT's `board_init()` and `kernel_init()`
6. RIOT's `kernel_init()` creates main thread using `main_trampoline()`
7. RIOT's `kernel_init()` starts threading by calling riot-rs-core `cpu_switch_context_exit()`
8. main thread gets scheduled, executing `main_trampoline()`
9. `main_trampoline()` calls `auto_init()`
10. `main_trampoline()` calls application `main()`
