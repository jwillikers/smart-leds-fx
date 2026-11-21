set shell := ["nu", "-c"]

default_example := "soft-blink-onboard"

default: build

alias b := build

# Build the examples, or a single example when one is given.
build example="" profile="dev":
    #!/usr/bin/env nu
    if ("{{ example }}" | is-empty) {
        ^cargo build --profile "{{ profile }}" --examples
    } else {
        ^cargo build --profile "{{ profile }}" --example "{{ example }}"
    }

alias f := flash

# Build and flash an example over USB with the UF2 bootloader using cargo-hf2.
flash example=default_example:
    ^cargo hf2 --release --example "{{ example }}"

alias r := run

# Build and run an example using the Cargo runner configured in .cargo/config.toml.
run example=default_example profile="dev":
    #!/usr/bin/env nu
    let example = if ("{{ example }}" | is-empty) { "{{ default_example }}" } else { "{{ example }}" }
    ^cargo run --profile "{{ profile }}" --example $example

alias p := package
alias pack := package

package:
    ^nix build ".#feather-m4-express"

alias u := update
alias up := update

update:
    nix flake update
    cargo update
