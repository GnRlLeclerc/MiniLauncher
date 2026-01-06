{
  description = "Fancy and minimal app launcher build with Slint";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
    }:
    let
      # Supported systems
      systems = [
        "aarch64-linux"
        "i686-linux"
        "x86_64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShell = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        pkgs.mkShell rec {
          packages = with pkgs; [
            slint-lsp
          ];
          buildInputs = with pkgs; [
            libGL
            libxkbcommon
            wayland
            fontconfig
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        }
      );
    };
}
