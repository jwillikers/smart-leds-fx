set shell := ["nu", "-c"]

default_example := "rgbw-neopixel-stick-soft-blink"

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
    let elf = $"target/riscv32imac-unknown-none-elf/($build_type)/examples/{{ example }}"
    ^rust-objcopy --output-target binary $elf $"($elf).bin"
    ^dfu-util --alt 0 --dfuse-address 0x08000000:leave --download $"($elf).bin"

alias r := run

# Build and run an example using the Cargo runner configured in .cargo/config.toml.
run example=default_example profile="dev":
    #!/usr/bin/env nu
    let example = if ("{{ example }}" | is-empty) { "{{ default_example }}" } else { "{{ example }}" }
    ^cargo run --profile "{{ profile }}" --example $example

alias p := package
alias pack := package

package:
    ^nix build ".#longan-nano"

alias u := update
alias up := update

update:
    nix flake update
    cargo update
