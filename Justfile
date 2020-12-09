# some settings you might want to change or override
export BOARD := "nrf52840dk"
export RIOTBASE := "target/RIOT"
export SCRIPTS := justfile_directory() + "/scripts"
host_build := "target/host"
RIOT_URL := "https://github.com/kaspar030/RIOT -b riot.rs"


# list of folders that "just test" should run "cargo test" in
tests := "src/riot-rs-rt src/riot-rs-core"

test-core PROFILE="release" TARGET="thumbv7m-none-eabi":
    @for test in {{tests}}; do \
       echo "Testing $test in {{PROFILE}} profile..."; \
       . env/lm3s6965evb.env ; \
       cargo -Zunstable-options test --manifest-path $test/Cargo.toml --features riot-rs-boards/lm3s6965evb --profile {{PROFILE}} \
            --target {{TARGET}}; \
    done

test: (test-core "debug") (test-core "release")

clone-riot:
    @[ "{{RIOTBASE}}" = "target/RIOT" ] && test -f {{RIOTBASE}}/.cloned || git clone {{RIOT_URL}} {{RIOTBASE}} && touch {{RIOTBASE}}/.cloned

# build dependency installation

install-reqs: install-toolchains install-c2rust clone-riot

install-toolchains:
    rustup target add thumbv7m-none-eabi
    rustup target add thumbv7em-none-eabi
    rustup target add thumbv7em-none-eabihf

create-host_build:
    mkdir -p {{host_build}}

install-c2rust: create-host_build
    @echo "Installing riot-sys compatible version of c2rust."
    @echo "WARNING: This uses *a lot* of memory!"

    rustup install nightly-2019-12-05
    rustup component add --toolchain nightly-2019-12-05 rustfmt rustc-dev
    cargo +nightly-2019-12-05 install --debug --git https://github.com/kaspar030/c2rust --branch for-riot c2rust


# some convenience targets

bin := invocation_directory()

board_bin_common := "source env/" + BOARD + ".env ; source " + bin + "/env ; cd " + bin + ";"

cargo CMD *ARGS:
    {{board_bin_common}} cargo {{CMD}} {{ARGS}}

build *ARGS: clone-riot
    @echo "Building {{bin}} for {{BOARD}}..."
    {{board_bin_common}} cargo build --features "$FEATURES" {{ARGS}}

run *ARGS: clone-riot
    @echo "Running {{bin}} for {{BOARD}}..."
    {{board_bin_common}} cargo run --features "$FEATURES" {{ARGS}}

cmd +CMD:
    {{board_bin_common}} {{CMD}}
