{
  inputs = {
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, naersk, nixpkgs, rust-overlay }:
    let
      overlays = [ (import rust-overlay) ];

      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system_: f rec {
        system = system_;

        pkgs = import nixpkgs {
          inherit overlays system;
        };

        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
          clippy = toolchain;
        };
      });
    in
    rec {
      packages = forEachSupportedSystem
        ({ system, pkgs, toolchain, naersk' }: {
          default = naersk'.buildPackage {
            pname = "wl_translation_window";
            version = "0.1.0";

            src = ./.;

            buildInputs = with pkgs; [ openssl libxkbcommon glib pango gdk-pixbuf graphene gtk4 gtk4-layer-shell ];
            nativeBuildInputs = with pkgs; [ pkg-config ];
          };
        });

      devShells = forEachSupportedSystem
        ({ system, pkgs, toolchain, naersk' }: {
          default =
            pkgs.mkShell
              {
                nativeBuildInputs = with pkgs;
                  [ toolchain taplo ] ++ packages.${system}.default.buildInputs ++ packages.${system}.default.nativeBuildInputs;
              };
        });
    };
}
