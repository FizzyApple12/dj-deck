{
  inputs,
  nixos-raspberrypi,
  ...
}:
nixos-raspberrypi.lib.nixosSystem {
  system = "aarch64-linux";
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
    ({lib, ...}: {
      time = {
        timeZone = "UTC";
      };

      networking = {
        hostName = "djs1";
      };

      boot = {
        loader = {
          raspberry-pi = {
            bootloader = "kernel";
          };

          timeout = 0;
        };

        tmp = {
          useTmpfs = true;
        };

        supportedFilesystems = {
          zfs = lib.mkForce false;
        };

        plymouth = {
          enable = true;
        };

        consoleLogLevel = 3;

        initrd = {
          verbose = false;
        };

        kernelParams = [
          "quiet"
          "rd.udev.log_level=3"
          "rd.systemd.show_status=auto"
        ];
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
        getty = {
          autologinUser = "dj";
        };

        openssh = {
          enable = false;
        };
      };

      security = {
        sudo = {
          enable = true;
          wheelNeedsPassword = false;
        };
      };

      nix = {
      	settings = {
          trusted-users = ["dj"];
        };
      };

      system = {
      	stateVersion = "26.11";
      };
    })
    ({ nixpkgs, ... }: let
      system = "aarch64-linux";
      pkgs = import nixpkgs { inherit system; };
      deck-application = (pkgs.callPackage ../package.nix {});
    in {
      environment = {
        systemPackages = [
          deck-application
          pkgs.cage
        ];
      };

      services = {
        pipewire = {
          enable = true;
        };

        cage = {
          enable = true;
          extraArguments = [
            "-d"
            "-m extend"
          ];
          program = "${deck-application.outPath}/bin/deck-application";
          user = "dj";
        };
      };

      security = {
        rtkit = {
          enable = true;
        };
      };
    })
  ];
}
