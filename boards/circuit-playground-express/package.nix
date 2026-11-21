{
  cargoArtifacts,
  commonArgs,
  craneLib,
}:
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;
  }
)
