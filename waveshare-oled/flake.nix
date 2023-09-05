{
  inputs.nixify.url = github:rvolosatovs/nixify;

  outputs = {nixify, ...}:
    with nixify.lib;
      rust.mkFlake {
        src = ./.;
        name = "waveshareoled";

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
            ];
          }
          devShells;
      };
}
