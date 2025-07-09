{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "https://flakehub.com/f/ipetkov/crane/0.20.tar.gz";
    devshell.url = "github:numtide/devshell";
  };

  outputs = { nixpkgs, rust-overlay, devshell, flake-utils, crane, ... }: 
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) devshell.overlays.default ];
      };

      toolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
        extensions = [ "rust-src" "rust-analyzer" ];
      });

      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
      commonArgs = {
        src = craneLib.cleanCargoSource ./.;
        strictDeps = true;
      };
      jj-watch = craneLib.buildPackage (commonArgs // {
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        postInstall = ''
          ln -T $out/bin/jj-watch $out/bin/jjw
        '';
      });
    in {
      devShell = pkgs.devshell.mkShell {
        packages = [ toolchain pkgs.gcc pkgs.lldb ];
        motd = "\n  Welcome to the {2}jj-watch{reset} shell.\n";
      };
      packages = {
        inherit jj-watch;
        default = jj-watch;
      };
    });
}
