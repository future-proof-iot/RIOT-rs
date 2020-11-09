tests := "riot-rs-rt riot-rs-core"

test-core PROFILE="release" TARGET="thumbv7m-none-eabi":
    @for test in {{tests}}; do \
       echo "Testing $test in {{PROFILE}} profile..."; \
       cargo -Zunstable-options test --manifest-path $test/Cargo.toml --features boards/lm3s6965evb --profile {{PROFILE}} \
            --target {{TARGET}}; \
    done

test: (test-core "debug") (test-core "release")
