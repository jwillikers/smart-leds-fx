{
  cargoArtifacts,
  commonArgs,
  craneLib,
  llvmPackages,
}:
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;

    nativeBuildInputs = (commonArgs.nativeBuildInputs or [ ]) ++ [ llvmPackages.bintools-unwrapped ];

    # Create a raw binary file for each example for flashing with a bootloader.
    postInstall = ''
      for example in $out/bin/*; do
        llvm-objcopy --output-target binary "$example" "$example.bin"
      done
    '';
  }
)
