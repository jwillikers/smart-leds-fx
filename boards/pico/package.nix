{
  cargoArtifacts,
  commonArgs,
  craneLib,
  elf2uf2-rs,
}:
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;

    nativeBuildInputs = (commonArgs.nativeBuildInputs or [ ]) ++ [ elf2uf2-rs ];

    # Create a UF2 file for each example, which can be copied to the RP2040 in BOOTSEL mode.
    postInstall = ''
      for example in $out/bin/*; do
        elf2uf2-rs "$example" "$example.uf2"
      done
    '';
  }
)
