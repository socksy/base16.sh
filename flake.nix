{
  description = "Base16/Base24 color scheme server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    schemes = {
      url = "github:tinted-theming/schemes";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, schemes }:
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

          postInstall = ''
            mkdir -p $out/share/base16-server/data
            ln -s ${schemes} $out/share/base16-server/data/schemes
          '';
        };
      });

      nixosModules.default = { pkgs, config, lib, ... }:
        let
          cfg = config.services.base16-server;
        in
        {
          options.services.base16-server.enable = lib.mkEnableOption "base16-server";

          config = lib.mkIf cfg.enable {
            systemd.services.base16-server = {
              wantedBy = [ "multi-user.target" ];
              serviceConfig = {
                ExecStart = "${self.packages.${pkgs.system}.default}/bin/base16-server";
                WorkingDirectory = "${self.packages.${pkgs.system}.default}/share/base16-server";
                Restart = "always";
                DynamicUser = true;
              };
              environment.RUST_LOG = "info";
            };
          };
        };
    };
}
