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

      systemd = {
        services = {
          dj-usb-mkdir = {
            enable = true;
            after = [ "multi-user.target" ];
            wantedBy = [ "default.target" ];
            description = "Ensure DJ USB mount directories exist";
            serviceConfig = {
              Type = "simple";
              ExecStart = pkgs.writeShellScript "dj-usb-mkdir.sh" ''
                ${pkgs.coreutils}/bin/mkdir -p /djusb/usb0 /djusb/usb1
                ${pkgs.coreutils}/bin/chown dj:users /djusb /djusb/usb0 /djusb/usb1
              '';
            };
          };
        };
      };

      services = {
        pipewire = {
          enable = true;
        };

        udev = { # ports usb 2-1, usb 5-1
          extraRules = ''
            SUBSYSTEM=="block", KERNELS=="2-1", ENV{ID_BUS}=="usb", ENV{ID_FS_USAGE}=="filesystem", ACTION=="add", RUN+="${pkgs.writeShellScript "udev-mount-usb0.sh" ''
            	${pkgs.systemd}/bin/systemd-mount --no-block /dev/$1 /djusb/usb0
            ''} $kernel"
            SUBSYSTEM=="block", KERNELS=="2-1", ENV{ID_BUS}=="usb", ENV{ID_FS_USAGE}=="filesystem", ACTION=="remove", RUN+="${pkgs.writeShellScript "udev-unmount-usb0.sh" ''
            	${pkgs.systemd}/bin/systemd-mount --no-block -u /djusb/usb0
            ''}"
            SUBSYSTEM=="block", KERNELS=="5-1", ENV{ID_BUS}=="usb", ENV{ID_FS_USAGE}=="filesystem", ACTION=="add", RUN+="${pkgs.writeShellScript "udev-mount-usb1.sh" ''
            	${pkgs.systemd}/bin/systemd-mount --no-block /dev/$1 /djusb/usb1
            ''} $kernel"
            SUBSYSTEM=="block", KERNELS=="5-1", ENV{ID_BUS}=="usb", ENV{ID_FS_USAGE}=="filesystem", ACTION=="remove", RUN+="${pkgs.writeShellScript "udev-unmount-usb1.sh" ''
            	${pkgs.systemd}/bin/systemd-mount --no-block -u /djusb/usb1
            ''}"
          '';
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
