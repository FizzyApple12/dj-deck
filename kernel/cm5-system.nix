{
  nixos-raspberrypi,
  lib,
  pkgs,
  ...
}: {
  imports = [
    ./configtxt.nix
    ./configtxt-config.nix
  ];

  nixpkgs.overlays = lib.mkBefore [
    nixos-raspberrypi.overlays.jemalloc-page-size-16k
    (final: super: {
      makeModulesClosure = x:
        super.makeModulesClosure (x // {allowMissing = true;});
    })
  ];

  boot.loader.raspberry-pi = lib.mkForce {
    enable = true;
    variant = "5";
    bootloader = lib.mkDefault "kernelboot";
    firmwarePackage = lib.mkDefault nixos-raspberrypi.packages.${pkgs.stdenv.hostPlatform.system}.raspberrypifw;
  };

  boot.kernelPackages = lib.mkDefault nixos-raspberrypi.packages.${pkgs.stdenv.hostPlatform.system}.linuxPackages_rpi5;
  boot.initrd.availableKernelModules = [
    "nvme"
    "xhci_pci"
    "usbhid"
    "usb_storage"
    "vc4"
    "pcie_brcmstb"
    "reset-raspberrypi"
  ];
  boot.consoleLogLevel = lib.mkDefault 7;
  boot.kernelParams = ["console=serial0,115200n8" "console=tty1"];

  hardware.raspberry-pi.config.all.options = {
    arm_64bit = {
      enable = true;
      value = true;
    };
    enable_uart = {
      enable = true;
      value = true;
    };
    avoid_warnings = {
      enable = lib.mkDefault true;
      value = lib.mkDefault true;
    };
  };

  hardware.enableRedistributableFirmware = true;

  environment.systemPackages = with pkgs; [
    raspberrypi-utils
  ];

  users.extraGroups = {
    gpio = {};
    i2c = {};
    input = {};
    plugdev = {};
    spi = {};
    video = {};
  };

  services.udev.packages = [
    pkgs.raspberrypi-udev-rules
  ];

  systemd.tmpfiles.packages = [
    pkgs.raspberrypi-udev-rules
  ];
}
