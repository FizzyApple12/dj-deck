{
  description = "DJ Deck System Image";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";

    flake-utils.url = "github:numtide/flake-utils";

    nixos-raspberrypi.url = "github:nvmd/nixos-raspberrypi/main";
  };

  nixConfig = {
    extra-substituters = [
      "https://nixos-raspberrypi.cachix.org"
    ];
    extra-trusted-public-keys = [
      "nixos-raspberrypi.cachix.org-1:4iMO9LXa8BqhU+Rpg6LQKiGa2lsNh/j2oiYLNOQ5sPI="
    ];
  };

  outputs = {
    flake-utils,
    nixos-raspberrypi,
    nixpkgs,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in rec {
        nixosConfigurations.djdeck = import ./deckos {
          inputs = inputs;
          nixos-raspberrypi = nixos-raspberrypi;
        };

        packages = {
          deckos = nixosConfigurations.djdeck.config.system.build.sdImage;
          deck-application = pkgs.callPackage ./package.nix {};
        };

        devShells.default = let
            qtEnv = with pkgs.qt6; env "qt-custom-${qtbase.version}" [
              qtdeclarative
              qtwayland
              qtshadertools
            ];
          in pkgs.mkShell rec {
          buildInputs = [
            pkgs.systemd
            pkgs.openssl

            pkgs.fontconfig
            pkgs.vulkan-loader
            pkgs.libglvnd

            pkgs.libxkbcommon
            pkgs.xorg.libxcb
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXi
            pkgs.xorg.libXrandr
            pkgs.xorg.libXxf86vm

            pkgs.wayland
            pkgs.wayland-protocols
            pkgs.wayland-scanner

            pkgs.alsa-lib
            pkgs.libjack2
            pkgs.pipewire
            pkgs.pipewire.jack

            qtEnv
          ];
          nativeBuildInputs = [
            pkgs.bash
            pkgs.git

            pkgs.pkg-config
            pkgs.cmake
            pkgs.rustup

            pkgs.clang
            pkgs.llvmPackages.bintools

            pkgs.yaml-language-server

            pkgs.rpiboot
            pkgs.minicom

            pkgs.qtcreator

            qtEnv
          ];

          RUSTC_VERSION = "nightly-2026-07-19";

          LIBCLANG_PATH = pkgs.lib.makeLibraryPath [pkgs.llvmPackages.libclang.lib];

          shellHook = ''
            export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
            export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/

            flashImage() {
              if [ "$#" -eq 0 ]; then
                echo "Please Specify a Block Device to Flash";
              else
                zstdcat result/sd-image/nixos-image-rpi5-kernel.img.zst | sudo dd of=$1 bs=100M status=progress
              fi
            }
          '';

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (buildInputs ++ nativeBuildInputs);
        };
      }
    );
}
