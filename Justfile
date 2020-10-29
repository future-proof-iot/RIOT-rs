tests := "riot-rs-core"

test-core PROFILE="release" TARGET="thumbv7m-none-eabi":
    for test in {{tests}}; do \
       cd $test; cargo -Zunstable-options test --features boards/lm3s6965evb --profile {{PROFILE}} \
            --target {{TARGET}}; \
    done

test: (test-core "debug") (test-core "release")
