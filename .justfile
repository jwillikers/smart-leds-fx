set shell := ["nu", "-c"]

default: build

alias c := check

check: && format
    #!/usr/bin/env nu
    ^yamllint .
    ^asciidoctor '**/*.adoc'
    ^lychee --cache **/*.html LICENSE-*
    ^nix flake check

alias b := build

# Build the examples for a board, or a single example when one is given.
build board="pico" example="" profile="dev":
    #!/usr/bin/env nu
    cd "boards/{{ board }}"
    if ("{{ example }}" | is-empty) {
        ^nix develop "../..#{{ board }}" --command cargo build --profile "{{ profile }}" --examples
    } else {
        ^nix develop "../..#{{ board }}" --command cargo build --profile "{{ profile }}" --example "{{ example }}"
    }

alias f := flash

# Flash an example to a board using the Nix package.
flash board="pico" example="":
    #!/usr/bin/env nu
    if ("{{ example }}" | is-empty) {
        ^nix run ".#{{ board }}-flash"
    } else {
        ^nix run ".#{{ board }}-flash" -- "{{ example }}"
    }

alias fmt := format

format:
    treefmt

alias r := run

# Build and run an example using the board's Cargo runner.
run board="pico" example="" profile="dev":
    #!/usr/bin/env nu
    cd "boards/{{ board }}"
    ^nix develop "../..#{{ board }}" --command just run "{{ example }}" "{{ profile }}"

alias p := package
alias pack := package

package board="pico":
    nix build ".#{{ board }}"

alias t := test

test:
    #!/usr/bin/env nu
    cd smart-leds-fx
    ^nix develop "..#smart-leds-fx" --command cargo test

alias u := update
alias up := update

update:
    #!/usr/bin/env nu
    ^nix run ".#update-nix-direnv"
    ^nix run ".#update-nixos-release"
    ^nix flake update
    cd "{{ justfile_directory() }}/smart-leds-fx"
    ^nix develop "..#smart-leds-fx" --command cargo update
    for board in (ls "{{ justfile_directory() }}/boards" | where type == dir | get name | path basename) {
        cd $"{{ justfile_directory() }}/boards/($board)"
        ^nix develop $"../..#($board)" --command cargo update
    }
