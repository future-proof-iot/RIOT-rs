tests := "riot_core/tests/*.rs"

test-core:
    for test in {{tests}}; do \
       cargo test -p riot_core --test $(basename $test .rs) --target thumbv7m-none-eabi; \
    done
