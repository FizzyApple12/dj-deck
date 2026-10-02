{
  lib,
  system,
  makeRustPlatform,
  pkg-config,
  systemd,
  openssl,
  cmake,
  clang,
  llvmPackages,
  fontconfig,
  vulkan-loader,
  libxkbcommon,
  xorg,
  wayland,
  wayland-protocols,
  wayland-scanner,
  alsa-lib,
  libjack2,
  pipewire,
}:
let
  rustToolchain = "nightly-2026-07-19";
  fenix = import (fetchTarball {
  	url = "https://github.com/nix-community/fenix/archive/2013c981829bd5e93db06749b5639450c81bece3.tar.gz";
   	sha256 = "sha256-ZNw7FDQm2PVOIyE9WaLFgK1QYw8vrOchxMYHzDDxlUY=";
  }) { inherit system; };
  fenixToolchain = fenix.fromToolchainName { name = rustToolchain; sha256 = "sha256-ziDE7hJrDjZINklZE8gmLwXjmCa2B1aR2Z97NUcA77Y="; };
  rustPlatform = makeRustPlatform {
  	rustc = fenixToolchain.rustc;
    cargo = fenixToolchain.cargo;
  };
in
rustPlatform.buildRustPackage {
  pname = "deck-application";
  version = "1.0.0";
  src = lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "rekordcrate-0.3.0" = "sha256-bOcjGdotVeLDn/0kqlLOR1t5lABuNk401Ky/oP2B4yI=";
      "collections-0.1.0" = "sha256-AAoSoA1jHMFu2x5xcg/DHj2xkc1zi0OG3UEMBXVMVWA=";
      "wasm_thread-0.3.3" = "sha256-+lRLCIk0S6Y5ORYjDKsYYHia2FtoSoh+rWkQh7mnPBE=";
      "zed-font-kit-0.14.1-zed" = "sha256-KXygi0olNQi5yM8eaJVykNDtbPMDjT+cWPBF8UrtXR4=";
      "zed-scap-0.0.8-zed" = "sha256-BihiQHlal/eRsktyf0GI3aSWsUCW7WcICMsC2Xvb7kw=";
    };
  };

  buildInputs = [
    systemd
    openssl

    fontconfig
    vulkan-loader

    libxkbcommon
    xorg.libxcb
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    xorg.libXxf86vm

    wayland
    wayland-protocols
    wayland-scanner

    alsa-lib
    libjack2
    pipewire
    pipewire.jack
  ];

  nativeBuildInputs = [
    pkg-config
    cmake

    clang
    llvmPackages.bintools
  ];

  RUSTC_VERSION = rustToolchain;

  LIBCLANG_PATH = lib.makeLibraryPath [llvmPackages.libclang.lib];

  meta = {
    description = "DJ Deck Application for Embedded Systems";
    homepage = "https://github.com/fizzyapple12/djdeck";
    maintainers = with lib.maintainers; [ fizzyapple12 ];
    mainProgram = "deck-application";
    platforms = lib.platforms.all;
  };
}
