{
  lib,
  rustPlatform,
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
}:
rustPlatform.buildRustPackage {
  pname = "deck-application";
  version = "1.0.0";
  cargoLock.lockFile = ./Cargo.lock;
  src = lib.cleanSource ./.;

  nativeBuildInputs = [
    pkg-config
    systemd
    openssl
    cmake
    clang
    llvmPackages.bintools

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
  ];

  RUSTC_VERSION = "nightly";

  meta = {
    description = "DJ Deck Application for Embedded Systems";
    homepage = "https://github.com/fizzyapple12/djdeck";
    maintainers = with lib.maintainers; [ fizzyapple12 ];
    mainProgram = "deck-application";
    platforms = lib.platforms.all;
  };
}
