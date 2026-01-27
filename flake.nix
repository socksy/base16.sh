{
  description = "Base16/Base24 color scheme server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "base16-server";
        version = "0.1.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
      };

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
