tests := "riot_core/tests/*.rs"

test-core PROFILE="release" TARGET="thumbv7m-none-eabi":
    for test in {{tests}}; do \
       cargo -Z unstable-options test -p riot_core --profile {{PROFILE}} \
            --test $(basename $test .rs) --target {{TARGET}}; \
    done

test: (test-core "debug") (test-core "release")
