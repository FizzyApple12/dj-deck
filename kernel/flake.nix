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
    self,
    flake-utils,
    nixpkgs,
    nixos-raspberrypi,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        # pkgs = nixpkgs.legacyPackages.${system};
        lib = nixpkgs.lib;
      in rec {
        nixosConfigurations = {
          djdeck = nixos-raspberrypi.lib.nixosSystemFull {
            specialArgs = inputs;
            modules = [
              {
                # Hardware specific configuration, see section below for a more complete
                # list of modules
                imports = with nixos-raspberrypi.nixosModules; [
                  raspberry-pi-5.base
                  raspberry-pi-5.page-size-16k
                  raspberry-pi-5.display-vc4
                ];
              }
              ({config, ...}: {
                # imports = with nixos-raspberrypi.nixosModules; [
                #   ./cm5-system.nix
                #   # raspberry-pi-5.display-vc4
                #   # raspberry-pi-5.page-size-16k
                #   # ./pi5-configtxt.nix
                # ];

                time.timeZone = "UTC";
                networking.hostName = "djdeck";

                # boot.loader.raspberry-pi.bootloader = lib.mkForce "kernel";
                # boot.loader.grub.enable = lib.mkForce false;
                # boot.loader.systemd-boot.enable = lib.mkForce false;
                boot.loader.generic-extlinux-compatible.enable = lib.mkForce false;

                security.polkit.enable = true;

                security.sudo = {
                  enable = true;
                  wheelNeedsPassword = false;
                };

                services.getty.autologinUser = "dj";

                users.users.dj = {
                  initialPassword = "thisIsMyMusic";
                  isNormalUser = true;
                  extraGroups = [
                    "wheel"
                    "networkmanager"
                    "gpio"
                    "i2c"
                    "input"
                    "plugdev"
                    "spi"
                    "video"
                  ];
                };

                services.openssh.enable = true;
                services.openssh.settings.PermitRootLogin = "yes";

                nix.settings.trusted-users = ["dj"];

                system.stateVersion = "25.11";
              })
            ];
          };
        };

        packages.default = nixosConfigurations.djdeck.config.system.build.images.sd-card;
      }
    );
}
