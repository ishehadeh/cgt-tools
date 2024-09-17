{
  description = "cgt-tools";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    pre-commit-hooks-nix = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixpkgs-stable.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
    };
    hercules-ci-effects = {
      url = "github:hercules-ci/hercules-ci-effects";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-parts.follows = "flake-parts";
    };
  };
  outputs = inputs @ {self, ...}:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.pre-commit-hooks-nix.flakeModule
        inputs.hercules-ci-effects.flakeModule
      ];

      # `nix flake show --impure` hack
      systems =
        if builtins.hasAttr "currentSystem" builtins
        then [builtins.currentSystem]
        else inputs.nixpkgs.lib.systems.flakeExposed;

      herculesCI.ciSystems = ["x86_64-linux"];
      hercules-ci.github-releases.files = [
        {
          label = "cgt-tools-x86_64-windows.zip";
          path = "${self.outputs.packages.x86_64-linux.cgt-tools-x86_64-windows-bundle}";
        }
      ];

      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        lib,
        system,
        ...
      }: let
        rustToolchain = pkgs.rust-bin.fromRustupToolchain {
          channel = "stable";
          components = ["rust-analyzer" "rust-src" "rustfmt" "rustc" "cargo"];
          targets = [
            "x86_64-unknown-linux-gnu"
            "x86_64-unknown-linux-musl"
          ];
        };

        pythonToolchain = "python311";

        hostPkgs = pkgs;

        mkCgtTools = {
          pkgs,
          SDL2 ? pkgs.SDL2,
        }:
          pkgs.rustPlatform.buildRustPackage {
            name = "cgt-tools";

            src = lib.cleanSourceWith {
              src = ./.;
              filter = name: type: let
                baseName = baseNameOf (toString name);
              in
                !((!lib.cleanSourceFilter name type)
                  || (baseName == "flake.lock")
                  || (lib.hasSuffix ".nix" baseName));
            };
            cargoLock.lockFile = ./Cargo.lock;

            cargoBuildFlags = ["-p cgt_gui -p cgt_cli"];

            nativeBuildInputs = [
              hostPkgs.pkg-config
            ];

            buildInputs = [
              pkgs.freetype
              SDL2
            ];

            doCheck = false;
          };
      in {
        _module.args.pkgs = import self.inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.rust-overlay.overlays.rust-overlay
          ];
        };

        pre-commit.settings = {
          src = ./.;
          hooks = {
            alejandra.enable = true;
            rustfmt.enable = true;
          };
          tools = {
            rustfmt = lib.mkForce rustToolchain;
            clippy = lib.mkForce rustToolchain;
          };
        };

        packages = {
          cgt-tools-x86_64-windows = mkCgtTools {
            pkgs = pkgs.pkgsCross.mingwW64;
          };

          cgt-tools-x86_64-windows-bundle =
            pkgs.runCommand "cgt-tools-x86_64-windows.zip" {
              nativeBuildInputs = [pkgs.zip];
            } ''
              cp -vLr ${self'.packages.cgt-tools-x86_64-windows}/bin/ ./cgt-tools-x86_64-windows
              zip -r cgt-tools-x86_64-windows.zip cgt-tools-x86_64-windows
              mv cgt-tools-x86_64-windows.zip $out
            '';

          cgt-tools = mkCgtTools {
            inherit pkgs;
          };
        };

        devShells.default = pkgs.mkShell {
          shellHook = ''
            ${config.pre-commit.installationScript}
            PATH=$PATH:$(pwd)/target/release
          '';

          nativeBuildInputs = [
            pkgs.${pythonToolchain}
            pkgs.${pythonToolchain}.pkgs.pip
            pkgs.alejandra
            pkgs.cargo-expand
            pkgs.cargo-flamegraph
            pkgs.cargo-leptos
            pkgs.cargo-machete
            pkgs.cargo-modules
            pkgs.cargo-nextest
            pkgs.cargo-semver-checks
            pkgs.cargo-tarpaulin
            pkgs.cargo-udeps
            pkgs.fd
            pkgs.graphviz
            pkgs.heaptrack
            pkgs.hyperfine
            pkgs.maturin
            pkgs.sage
            pkgs.texlive.combined.scheme-full
            pkgs.trunk

            pkgs.pkg-config
            pkgs.freetype
            pkgs.SDL2

            rustToolchain
            pkgs.wineWow64Packages.unstableFull
          ];
        };
        formatter = pkgs.alejandra;
      };
    };
}
