{
  lib,
  rustPlatform
}:
rustPlatform.buildRustPackage {
  pname = "deck-application";
  version = "1.0.0";
  cargoLock.lockFile = ./Cargo.lock;
  src = lib.cleanSource ./.;

  meta = {
    description = "DJ Deck Application for Embedded Systems";
    homepage = "https://github.com/fizzyapple12/djdeck";
    maintainers = with lib.maintainers; [ fizzyapple12 ];
    mainProgram = "deck-application";
    platforms = lib.platforms.all;
  };
}
