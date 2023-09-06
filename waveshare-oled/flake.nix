{
  inputs.nixify.url = github:rvolosatovs/nixify;
  inputs.wash.url = github:wasmcloud/wash/v0.20.1;

  outputs = {
    nixify,
    wash,
    ...
  }:
    with nixify.lib;
      rust.mkFlake {
        src = ./.;
        name = "waveshareoled";

        overlays = [
          wash.overlays.default
        ];

        build.packages = [
          "waveshareoled-provider"
        ];

        doCheck = false; # testing is performed in checks via `nextest`

        buildOverrides = {
          pkgs,
          pkgsCross ? pkgs,
          ...
        }: {nativeBuildInputs ? [], ...}:
          with pkgsCross; {
            nativeBuildInputs =
              nativeBuildInputs
              ++ [
                pkgs.protobuf # prost build dependency
              ];
          };

        withDevShells = {
          devShells,
          pkgs,
          ...
        }:
          extendDerivations {
            buildInputs = [
              pkgs.protobuf # prost build dependency
              pkgs.wash
            ];
          }
          devShells;
      };
}
