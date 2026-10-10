{
  description = "Sursface, a cross-platform rendering library";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

    tomers = {
      url = "github:boralg/tomers";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      tomers,
      ...
    }:
    tomers.inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        targetPlatforms =
          let
            buildFilePatterns = [ ".*/assets/.*" ];
            toolchainPackages =
              fenixPkgs: crossFenixPkgs: with fenixPkgs; [
                latest.rustfmt
                stable.rustc
                stable.cargo
                crossFenixPkgs.stable.rust-src
                stable.rust-std
              ];

            wasm-bindgen-cli = pkgs.wasm-bindgen-cli.overrideAttrs (old: rec {
              version = "0.2.114";
              src = pkgs.fetchCrate {
                pname = "wasm-bindgen-cli";
                inherit version;
                hash = "sha256-xrCym+rFY6EUQFWyWl6OPA+LtftpUAE5pIaElAIVqW0=";
              };
              cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
                inherit src;
                name = "wasm-bindgen-cli-${version}-vendor";
                hash = "sha256-Z8+dUXPQq7S+Q7DWNr2Y9d8GMuEdSnq00quUR0wDNPM=";
              };
            });
          in
          [
            {
              system = "x86_64-unknown-linux-gnu";
              arch = "x86_64-linux";
              depsBuild = with pkgs; [
                patchelf
              ];
              postInstall = crateName: ''
                  find $out -type f -exec sh -c '
                  if file "$1" | grep -q "ELF .* executable"; then
                    patchelf --set-interpreter "/lib64/ld-linux-x86-64.so.2" "$1"
                  fi
                ' sh {} \;
              '';
              inherit buildFilePatterns;
              inherit toolchainPackages;
              env = {
                LD_LIBRARY_PATH =
                  with pkgs;
                  lib.makeLibraryPath [
                    vulkan-loader
                    libGL

                    libxkbcommon
                    wayland
                    libx11
                    libxrandr
                    libxrender
                    libxcursor
                    libxcb
                    libxi
                    libxrandr
                  ];
              };
            }
            {
              system = "x86_64-pc-windows-gnu";
              arch = "x86_64-windows";
              depsBuild = with pkgs.pkgsCross; [
                mingwW64.stdenv.cc
              ];
              targetLibs = with pkgs.pkgsCross; [
                mingwW64.windows.pthreads
              ];
              inherit buildFilePatterns;
              inherit toolchainPackages;
              env = {
                # fixes issues related to libring
                TARGET_CC = with pkgs.pkgsCross; "${mingwW64.stdenv.cc}/bin/${mingwW64.stdenv.cc.targetPrefix}cc";

                #fixes issues related to openssl
                OPENSSL_DIR = "${pkgs.openssl.dev}";
                OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
                OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";
              };
            }
            {
              system = "wasm32-unknown-unknown";
              arch = "wasm32-unknown";
              depsBuild = with pkgs; [ wasm-bindgen-cli ];
              postInstall = crateName: ''
                mkdir $out/bindgen
                for wasm in $out/lib/*.wasm; do
                  wasm-bindgen "$wasm" --out-dir $out/bindgen --web
                done
              '';

              inherit buildFilePatterns;
              inherit toolchainPackages;
              env = {
                packages = with pkgs; [ wasm-bindgen-cli ];
              };
            }
          ];
        tomersLib = tomers.libFor system targetPlatforms;
      in
      rec {
        packagesForEachPlatform = tomersLib.packagesForEachPlatform;
        devShellsForEachPlatform = tomersLib.devShellsForEachPlatform;

        packages = packagesForEachPlatform ./.;
        devShells = devShellsForEachPlatform ./.;
      }
    );
}
