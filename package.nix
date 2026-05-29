{ pkgs, crane, ... }: let

  # define the toolchain
  toolchain = p: p.rust-bin.selectLatestNightlyWith (toolchain: toolchain.minimal.override {
    extensions = [ "rustfmt" "clippy" ];
  });

  # build crane
  craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
  # common arguments for the package
  commonArgs = {
    src = craneLib.cleanCargoSource ./.;
    strictDeps = true;
  };
  # build all the deps
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

in
  craneLib.buildPackage (commonArgs // {
    # use the deps
    inherit cargoArtifacts;
    # symlink it as jjw
    postInstall = ''
      ln -T $out/bin/jj-watch $out/bin/jjw
    '';
  })
