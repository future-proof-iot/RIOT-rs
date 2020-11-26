# some settings you might want to change
host_build := "target/host"

# testing
tests := "riot-rs-rt riot-rs-core"

test-core PROFILE="release" TARGET="thumbv7m-none-eabi":
    @for test in {{tests}}; do \
       echo "Testing $test in {{PROFILE}} profile..."; \
       cargo -Zunstable-options test --manifest-path $test/Cargo.toml --features boards/lm3s6965evb --profile {{PROFILE}} \
            --target {{TARGET}}; \
    done

test: (test-core "debug") (test-core "release")

# build dependency installation

install-reqs: install-toolchains install-c2rust

install-toolchains:
    rustup target add thumbv7m-none-eabi
    rustup target add thumbv7em-none-eabihf

create-host_build:
    mkdir -p {{host_build}}

install-c2rust: create-host_build
    @echo "Installing riot-sys compatible version of c2rust."
    @echo "WARNING: This uses *a lot* of memory!"

    rustup install nightly-2019-12-05
    rustup component add --toolchain nightly-2019-12-05 rustfmt rustc-dev
    cargo +nightly-2019-12-05 install --debug --git https://github.com/kaspar030/c2rust --branch for-riot c2rust
