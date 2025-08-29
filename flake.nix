{
  description = "BoxedNix";

  inputs = {
    nixpkgs.url = "github:Nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }: let
    dependencies = pkgs: let
      inherit (pkgs) clang;
    in {
      nativeBuildInputs = [clang];
    };
  in
    {
      overlays.default = final: _prev: let
        inherit (final.rustPlatform) buildRustPackage;
      in {
        boxednix = buildRustPackage {
          pname = "boxednix";
          version = "0.1.0";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;

          inherit ((dependencies final)) nativeBuildInputs;
        };
      };
    }
    // flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [self.overlays.default];
      };
    in {
      packages.default = pkgs.boxednix;
      devShells = {
        default = pkgs.mkShell {
          packages = with pkgs; [rage reuse pre-commit];
          inherit ((dependencies pkgs)) nativeBuildInputs;
          shellHook = ''
            echo "[devShell]" reuse + pre-commit environment active"
            export REUSE_LICENSES_DIRECTORY=${pkgs.reuse}/share/licenses
          '';
        };
      };
    });
}
