{
  config,
  pkgs,
  lib,
  ...
}: {
  hardware.raspberry-pi.config = {
    all = {
      options = {
        camera_auto_detect = {
          enable = lib.mkDefault true;
          value = lib.mkDefault true;
        };

        display_auto_detect = {
          enable = lib.mkDefault true;
          value = lib.mkDefault true;
        };

        max_framebuffers = {
          enable = lib.mkDefault true;
          value = lib.mkDefault 2;
        };

        # Don't have the firmware create an initial video= setting in cmdline.txt.
        # Use the kernel's default instead.
        disable_fw_kms_setup = {
          enable = lib.mkDefault true;
          value = lib.mkDefault true;
        };

        # Disable compensation for displays with overscan
        disable_overscan = {
          enable = lib.mkDefault true;
          value = lib.mkDefault true;
        };

        # Run as fast as firmware / board allows
        arm_boost = {
          enable = lib.mkDefault true;
          value = lib.mkDefault true;
        };

        enable_uart = {
          enable = true;
          value = true;
        };

        uart_2ndstage = {
          enable = true;
          value = true;
        };
      };
      base-dt-params = {
        # Uncomment some or all of these to enable the optional hardware interfaces
        # i2c_arm = {
        #   enable = true;
        #   value = "on";
        # };
        i2s = {
          enable = true;
          value = "on";
        };
        # spi = {
        #   enable = true;
        #   value = "on";
        # };
        pciex1 = {
          enable = true;
          value = "on";
        };
        pciex1_gen = {
          enable = true;
          value = "3";
        };
        audio = {
          enable = true;
          value = "on";
        };
      };
      dt-overlays = {
        vc4-kms-v3d = {
          enable = lib.mkDefault true;
          params = {};
        };
      };
    };
    cm4 = {
      options = {
        otg_mode = {
          enable = lib.mkDefault true;
          value = lib.mkDefault true;
        };
      };
    };
    cm5 = {
      dt-overlays = {
        dwc2 = {
          enable = lib.mkDefault true;
          params = {
            dr_mode = {
              enable = true;
              value = "host";
            };
          };
        };
      };
    };
  };
}
