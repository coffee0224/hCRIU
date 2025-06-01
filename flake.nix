{
  description = "A Nix flake for Rust development with CRIU tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
    systems = [ "x86_64-linux" ];
    forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f {
      inherit system;
      pkgs = import nixpkgs { inherit system; };
    });
  in {
    devShells = forAllSystems ({ pkgs, ... }: {
      default = pkgs.mkShell {
        packages = [
          pkgs.rustup
          pkgs.cargo
          pkgs.criu
        ];
        shellHook = ''
          echo "Welcome to the Rust development environment with CRIU!"
        '';
      };
    });
  };
}