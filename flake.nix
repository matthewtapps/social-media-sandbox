{
  description = "eframe devShell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        inherit (pkgs) lib stdenv;
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        toolchain = pkgs.rust-bin.beta.latest.minimal.override {
          targets = [
            "wasm32-unknown-unknown"
            "x86_64-unknown-linux-gnu"
          ];
        };
      in
      with pkgs;
      {
        devShells.default = mkShell rec {
          buildInputs = [
            rustc
            cargo
            trunk

            # misc. libraries
            openssl
            pkg-config
            lld
            vulkan-loader

            # GUI libs
            libxkbcommon
            libGL
            fontconfig

            # wayland libraries
            wayland

            # x11 libraries
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11
            xorg.libxcb

            rust-analyzer-unwrapped
            rustfmt
            clippy
          ];

          NIX_LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
          NIX_LD = lib.fileContents "${stdenv.cc}/nix-support/dynamic-linker";

          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
        };
      }
    );
}
