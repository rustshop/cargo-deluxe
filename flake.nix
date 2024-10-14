{
  description = "Flakebox Project template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flakebox.url = "github:rustshop/flakebox";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      flakebox,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        projectName = "cargo-deluxe";

        flakeboxLib = flakebox.lib.${system} {
          config = {
            github.ci.buildOutputs = [ ".#ci.${projectName}" ];
            toolchain.channel = "latest";
          };
        };

        buildPaths = [
          "Cargo.toml"
          "Cargo.lock"
          "cargo-deluxe"
          "bin-intercept"
        ];

        buildSrc = flakeboxLib.filterSubPaths {
          root = builtins.path {
            name = projectName;
            path = ./.;
          };
          paths = buildPaths;
        };

        multiBuild = (flakeboxLib.craneMultiBuild { }) (
          craneLib':
          let
            craneLib = (
              craneLib'.overrideArgs {
                pname = projectName;
                src = buildSrc;
                nativeBuildInputs = [ ];
              }
            );
          in
          {
            ${projectName} = craneLib.buildPackage { };

            "cargo" = flakeboxLib.pickBinary {
              pkg = craneLib.buildPackage { };
              bin = "cargo";
            };
            "rustc" = flakeboxLib.pickBinary {
              pkg = craneLib.buildPackage { };
              bin = "rustc";
            };
          }
        );
      in
      {
        packages = {
          default = multiBuild.${projectName};
          cargo = multiBuild.cargo;
          rustc = multiBuild.rustc;
        };

        legacyPackages = multiBuild;

        devShells = flakeboxLib.mkShells { };
      }
    );
}
