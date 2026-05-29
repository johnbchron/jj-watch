{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "https://flakehub.com/f/ipetkov/crane/0.20.tar.gz";
    devshell.url = "github:numtide/devshell";
  };

  outputs = { nixpkgs, rust-overlay, devshell, flake-utils, crane, ... }: let
    # define jj-watch in an overlay
    overlay = prev: final: let
      prev' = prev.extend (import rust-overlay);
    in {
      jj-watch = prev'.callPackage ./package.nix { inherit crane; };
    };

    per-system = flake-utils.lib.eachDefaultSystem (system: let
      # jj-watch on the given system, through the overlay
      jj-watch = (import nixpkgs { inherit system; overlays = [ overlay ]; }).jj-watch;

      # pkgs for devshell
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) devshell.overlays.default ];
      };
      dev-toolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
        extensions = [ "rust-src" "rust-analyzer" ];
      });
      nativeBuildInputs = [
        dev-toolchain
        pkgs.gcc
      ] ++ (pkgs.lib.optionals pkgs.stdenv.isDarwin [
        pkgs.libiconv
      ]);
    in {
      devShell = pkgs.mkShell {
        inherit nativeBuildInputs;
      };
      packages = {
        inherit jj-watch;
        default = jj-watch;
      };
    });
  in {
    overlays = {
      default = overlay;
    };
  } // per-system;
}
