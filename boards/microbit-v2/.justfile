set shell := ["nu", "-c"]

default_example := "soft-blink-neopixel-strip"

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

flash example=default_example profile="release": (build example profile)
    #!/usr/bin/env nu
    let build_type = if "{{ profile }}" == "dev" { "debug" } else { "{{ profile }}" }
    let elf = $"target/thumbv7em-none-eabihf/($build_type)/examples/{{ example }}"
    ^probe-rs run --chip nRF52833_xxAA --protocol swd $elf

alias r := run

# Build and run an example using the Cargo runner configured in .cargo/config.toml.
run example=default_example profile="dev":
    #!/usr/bin/env nu
    let example = if ("{{ example }}" | is-empty) { "{{ default_example }}" } else { "{{ example }}" }
    ^cargo run --profile "{{ profile }}" --example $example

alias p := package
alias pack := package

package:
    ^nix build ".#microbit-v2"

alias u := update
alias up := update

update:
    nix flake update
    cargo update
