{pkgs ? import <nixpkgs> {}}:
with pkgs; pkgs.mkShell rec {
  nativeBuildInputs = [
    rustup
    pkg-config
    vulkan-loader
    vulkan-tools
    vulkan-validation-layers
    vulkan-headers
    cmake
    fontconfig
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    shaderc
    glslang
  ];
  
  SHADERC_LIB_DIR="${pkgs.shaderc.lib}/lib";

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
