{
  inputs = {
    advisory-db = {
      flake = false;
      url = "github:rustsec/advisory-db";
    };
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    nix-update-scripts = {
      url = "github:jwillikers/nix-update-scripts";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
        pre-commit-hooks.follows = "pre-commit-hooks";
        treefmt-nix.follows = "treefmt-nix";
      };
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
      url = "github:oxalica/rust-overlay";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      # deadnix: skip
      self,
      advisory-db,
      crane,
      flake-utils,
      nix-update-scripts,
      nixpkgs,
      pre-commit-hooks,
      rust-overlay,
      treefmt-nix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        defaultBoard = "pico";
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit overlays system;
        };
        inherit (pkgs) lib;
        commonNativeBuildInputs = with pkgs; [
          asciidoctor
          fish
          just
          lychee
          nil
          nushell
        ];
        pre-commit = pre-commit-hooks.lib.${system}.run (
          import ./pre-commit-hooks.nix { inherit pkgs treefmtEval; }
        );
        treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
        commonDevShellPackages = [
          treefmtEval.config.build.wrapper
        ]
        # Make formatters available for IDE's.
        ++ (lib.attrValues treefmtEval.config.build.programs)
        ++ commonNativeBuildInputs
        ++ pre-commit.enabledPackages;

        # The smart-leds-fx library, which is used by every board.
        library = rec {
          craneLib = (crane.mkLib pkgs).overrideToolchain (
            p: p.rust-bin.fromRustupToolchainFile ./smart-leds-fx/rust-toolchain.toml
          );
          commonArgs = {
            src = craneLib.cleanCargoSource ./smart-leds-fx;
            strictDeps = true;
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          devShell = craneLib.devShell {
            packages = commonDevShellPackages;
            inherit (pre-commit) shellHook;
          };
        };

        # Create the derivations for a board.
        #
        # Each board is a separate Cargo package in the boards directory with its own toolchain and lock file.
        # The examples for each board are built and installed in the bin directory of the package.
        # Boards depend on the smart-leds-fx library through a relative path.
        # So, the source includes both the board's directory and the library.
        mkBoard =
          {
            # The name of the board's directory in the boards directory.
            name,
            # Arguments for Crane which are specific to this board.
            extraArgs ? { },
            # Additional packages for the development shell.
            devShellPackages ? [ ],
            # Takes the board's package and returns an attribute set of flake apps.
            mkApps ? _: { },
            # Allow additional attributes, such as the defaultExample used by mkApps.
            ...
          }:
          rec {
            boardDirectory = ./boards + "/${name}";
            craneLib = (crane.mkLib pkgs).overrideToolchain (
              p: p.rust-bin.fromRustupToolchainFile (boardDirectory + "/rust-toolchain.toml")
            );
            src = lib.cleanSourceWith {
              src = ./.;
              filter =
                path: type:
                let
                  relativePath = lib.removePrefix (toString ./. + "/") (toString path);
                  inBoard = lib.hasPrefix "boards/${name}" relativePath;
                  inLibrary = lib.hasPrefix "smart-leds-fx" relativePath;
                in
                # Keep the parent directories so that the filter descends into them.
                relativePath == "boards"
                || (
                  (inBoard || inLibrary)
                  && (
                    type == "directory"
                    || (craneLib.filterCargoSources path type)
                    # Don't remove linker scripts such as memory.x from the sources.
                    || (lib.hasSuffix ".x" path)
                  )
                );
            };
            commonArgs = {
              inherit src;
              pname = "smart-leds-fx-${name}";
              version = "0.1.0";
              strictDeps = true;
              doCheck = false;

              cargoLock = boardDirectory + "/Cargo.lock";
              cargoToml = boardDirectory + "/Cargo.toml";
              # Build the board's package from its own directory, so that Cargo picks up its .cargo/config.toml.
              postUnpack = ''
                cd $sourceRoot/boards/${name}
                sourceRoot="."
              '';
              cargoExtraArgs = "--examples";
            }
            // extraArgs;
            # Building the dependencies separately requires replacing the sources with dummy files.
            # The dummy examples can't be linked for embedded targets without the runtime crates.
            # So, the dependencies are built along with the board's examples.
            cargoArtifacts = null;
            package = pkgs.callPackage (boardDirectory + "/package.nix") {
              inherit cargoArtifacts commonArgs craneLib;
            };
            apps = mkApps package;
            devShell = craneLib.devShell {
              packages = devShellPackages ++ (extraArgs.nativeBuildInputs or [ ]) ++ commonDevShellPackages;
              inherit (pre-commit) shellHook;
            };
            checks = {
              "${name}-clippy" = craneLib.cargoClippy (
                commonArgs
                // {
                  inherit cargoArtifacts;
                  cargoClippyExtraArgs = "-- --deny warnings";
                }
              );
              # Audit dependencies
              "${name}-audit" = craneLib.cargoAudit {
                inherit advisory-db src;
                cargoAuditExtraArgs = "--file boards/${name}/Cargo.lock --ignore yanked";
              };
              # Audit licenses
              "${name}-deny" = craneLib.cargoDeny (
                commonArgs
                // {
                  cargoDenyChecks = "--config ${./deny.toml} bans licenses sources";
                  # Crane passes these arguments to Cargo before the deny subcommand.
                  cargoExtraArgs = "";
                }
              );
            };
          };

        # Write a Nushell script which takes the name of an example as an argument.
        writeExampleScript =
          name: defaultExample: body:
          builtins.toString (
            pkgs.writers.writeNu name ''
              def main [example: string = "${defaultExample}"] {
                ${body}
              }
            ''
          );

        mkProbeRsApp = chip: name: defaultExample: package: {
          type = "app";
          program = writeExampleScript "run-probe-rs-${name}" defaultExample ''
            ^${lib.getExe pkgs.probe-rs-tools} run --chip ${chip} --protocol swd $"${package}/bin/($example)"
          '';
        };

        mkRp2040Apps = name: defaultExample: package: {
          flash = {
            type = "app";
            program = writeExampleScript "flash-elf2uf2-rs-${name}" defaultExample ''
              ^${lib.getExe pkgs.elf2uf2-rs} --deploy $"${package}/bin/($example)"
            '';
          };
          run = mkProbeRsApp "RP2040" name defaultExample package;
        };

        boards = {
          circuit-playground-bluefruit = mkBoard rec {
            name = "circuit-playground-bluefruit";
            defaultExample = "soft-blink-onboard";
            devShellPackages = with pkgs; [
              cargo-binutils
              probe-rs-tools
            ];
            mkApps = package: {
              flash = mkProbeRsApp "nRF52840_xxAA" name defaultExample package;
            };
          };
          circuit-playground-express = mkBoard rec {
            name = "circuit-playground-express";
            defaultExample = "soft-blink-onboard";
            devShellPackages = with pkgs; [
              cargo-hf2
              probe-rs-tools
            ];
            mkApps = package: {
              flash = mkProbeRsApp "ATSAMD21G18A" name defaultExample package;
            };
          };
          esp32-c3-devkitc-02 = mkBoard rec {
            name = "esp32-c3-devkitc-02";
            defaultExample = "rgbw-neopixel-stick-high-low-soft-blink";
            devShellPackages = with pkgs; [
              espflash
            ];
            mkApps = package: {
              flash = {
                type = "app";
                program = writeExampleScript "flash-espflash-${name}" defaultExample ''
                  ^${lib.getExe pkgs.espflash} flash --monitor --chip esp32c3 $"${package}/bin/($example)"
                '';
              };
            };
          };
          feather-m4-express = mkBoard rec {
            name = "feather-m4-express";
            defaultExample = "soft-blink-onboard";
            devShellPackages = with pkgs; [
              cargo-hf2
              probe-rs-tools
            ];
            mkApps = package: {
              flash = mkProbeRsApp "ATSAMD51J19A" name defaultExample package;
            };
          };
          ft232h = mkBoard rec {
            name = "ft232h";
            defaultExample = "soft-blink-rgbw-neopixel-stick";
            extraArgs = {
              buildInputs = with pkgs; [
                libftdi1
                libusb1
              ];
              nativeBuildInputs = with pkgs; [
                pkg-config
              ];
            };
            devShellPackages = with pkgs; [
              libftdi1
              libusb1
            ];
            mkApps = package: {
              # The FT232H is controlled directly from the host, so there is nothing to flash.
              flash = {
                type = "app";
                program = writeExampleScript "run-${name}" defaultExample ''
                  ^$"${package}/bin/($example)"
                '';
              };
            };
          };
          longan-nano = mkBoard rec {
            name = "longan-nano";
            defaultExample = "rgbw-neopixel-stick-soft-blink";
            devShellPackages = with pkgs; [
              cargo-binutils
              dfu-util
              probe-rs-tools
            ];
            mkApps = package: {
              flash = {
                type = "app";
                program = writeExampleScript "flash-dfu-util-${name}" defaultExample ''
                  ^${lib.getExe' pkgs.dfu-util "dfu-util"} --alt 0 --dfuse-address 0x08000000:leave --download $"${package}/bin/($example).bin"
                '';
              };
            };
          };
          microbit-v2 = mkBoard rec {
            name = "microbit-v2";
            defaultExample = "soft-blink-neopixel-strip";
            devShellPackages = with pkgs; [
              probe-rs-tools
            ];
            mkApps = package: {
              flash = mkProbeRsApp "nRF52833_xxAA" name defaultExample package;
            };
          };
          pico = mkBoard rec {
            name = "pico";
            defaultExample = "rgbw-neopixel-strand-marching-candy-cane";
            extraArgs.nativeBuildInputs = with pkgs; [
              flip-link
            ];
            devShellPackages = with pkgs; [
              elf2uf2-rs
              probe-rs-tools
            ];
            mkApps = mkRp2040Apps name defaultExample;
          };
          qt-py-rp2040 = mkBoard rec {
            name = "qt-py-rp2040";
            defaultExample = "rgb-neopixel-strand-high-low-soft-blink";
            devShellPackages = with pkgs; [
              elf2uf2-rs
              probe-rs-tools
            ];
            mkApps = mkRp2040Apps name defaultExample;
          };
        };
      in
      {
        # Flake apps must be a flat attribute set, so prefix each board's apps with the board name.
        apps =
          lib.concatMapAttrs (
            board: value: lib.mapAttrs' (name: lib.nameValuePair "${board}-${name}") value.apps
          ) boards
          // {
            inherit (nix-update-scripts.apps.${system}) update-nix-direnv update-nixos-release;
            default = self.apps.${system}."${defaultBoard}-flash";
          };
        checks = lib.concatMapAttrs (_board: value: value.checks) boards // {
          smart-leds-fx-clippy = library.craneLib.cargoClippy (
            library.commonArgs
            // {
              inherit (library) cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
          smart-leds-fx-doc = library.craneLib.cargoDoc (
            library.commonArgs
            // {
              inherit (library) cargoArtifacts;
            }
          );
          smart-leds-fx-test = library.craneLib.cargoTest (
            library.commonArgs
            // {
              inherit (library) cargoArtifacts;
            }
          );
          smart-leds-fx-audit = library.craneLib.cargoAudit {
            inherit advisory-db;
            inherit (library.commonArgs) src;
            cargoAuditExtraArgs = "--ignore yanked";
          };
          smart-leds-fx-deny = library.craneLib.cargoDeny {
            inherit (library.commonArgs) src;
            cargoDenyChecks = "--config ${./deny.toml} bans licenses sources";
          };
        };
        devShells = lib.mapAttrs (_board: value: value.devShell) boards // {
          smart-leds-fx = library.devShell;
          default = pkgs.mkShell {
            nativeBuildInputs = commonDevShellPackages;
            inherit (pre-commit) shellHook;
          };
        };
        formatter = treefmtEval.config.build.wrapper;
        packages = lib.mapAttrs (_board: value: value.package) boards // {
          default = self.packages.${system}.${defaultBoard};
        };
      }
    );
}
