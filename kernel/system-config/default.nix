{
  inputs,
  nixos-raspberrypi,
  ...
}:
nixos-raspberrypi.lib.nixosSystem {
  specialArgs = inputs;
  modules = [
    ({...}: {
      imports = with nixos-raspberrypi.nixosModules; [
        sd-image
        raspberry-pi-5.base
        raspberry-pi-5.page-size-16k
        raspberry-pi-5.display-vc4
        ./configtxt.nix
      ];
    })
    ({...}: {
      time.timeZone = "UTC";
      networking.hostName = "djdeck";

      boot = {
        loader.raspberry-pi.bootloader = "kernel";
        tmp.useTmpfs = true;
      };

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

      services = {
        getty.autologinUser = "dj";
        openssh = {
          enable = true;
          settings.PermitRootLogin = "yes";
        };
      };

      security = {
        sudo = {
          enable = true;
          wheelNeedsPassword = false;
        };
      };

      nix.settings.trusted-users = ["dj"];

      system.stateVersion = "25.11";
    })
  ];
}
