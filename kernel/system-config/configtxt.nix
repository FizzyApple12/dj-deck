{lib, ...}: {
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

        disable_fw_kms_setup = {
          enable = lib.mkDefault true;
          value = lib.mkDefault true;
        };

        disable_overscan = {
          enable = lib.mkDefault true;
          value = lib.mkDefault true;
        };

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
