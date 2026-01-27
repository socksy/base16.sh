{
  description = "Base16/Base24 color scheme server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (pkgs: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "base16-server";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      });

      nixosModules.default = { pkgs, ... }: {
        systemd.services.base16-server = {
          wantedBy = [ "multi-user.target" ];
          serviceConfig = {
            ExecStart = "${self.packages.${pkgs.system}.default}/bin/base16-server";
            WorkingDirectory = "${self}";
            Restart = "always";
            DynamicUser = true;
          };
          environment.RUST_LOG = "info";
        };
      };
    };
}
