{
  description = "TwintailLauncher - multi-platform launcher for anime games";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };
    in
    {
      packages.${system} = rec {
        twintaillauncher = pkgs.callPackage ./nix/package.nix { };
        default = twintaillauncher;
      };

      apps.${system}.default = {
        type = "app";
        program = "${self.packages.${system}.default}/bin/twintaillauncher";
      };

      overlays.default = final: _prev: {
        twintaillauncher = final.callPackage ./nix/package.nix { };
      };
    };
}
