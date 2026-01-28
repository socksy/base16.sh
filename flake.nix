{
  description = "Base16/Base24 color scheme server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    schemes = { url = "github:tinted-theming/schemes"; flake = false; };
    base16-dunst = { url = "github:tinted-theming/base16-dunst"; flake = false; };
    base16-emacs = { url = "github:tinted-theming/base16-emacs"; flake = false; };
    base16-gtk-flatcolor = { url = "github:tinted-theming/base16-gtk-flatcolor"; flake = false; };
    base16-helix = { url = "github:tinted-theming/base16-helix"; flake = false; };
    base16-hexchat = { url = "github:tinted-theming/base16-hexchat"; flake = false; };
    base16-i3 = { url = "github:tinted-theming/base16-i3"; flake = false; };
    base16-jetbrains = { url = "github:tinted-theming/base16-jetbrains"; flake = false; };
    base16-kakoune = { url = "github:tinted-theming/base16-kakoune"; flake = false; };
    base16-kdeplasma = { url = "github:tinted-theming/base16-kdeplasma"; flake = false; };
    base16-polybar = { url = "github:tinted-theming/base16-polybar"; flake = false; };
    base16-qutebrowser = { url = "github:tinted-theming/base16-qutebrowser"; flake = false; };
    base16-rofi = { url = "github:tinted-theming/base16-rofi"; flake = false; };
    base16-scintillua = { url = "github:tinted-theming/base16-scintillua"; flake = false; };
    base16-sioyek = { url = "github:tinted-theming/base16-sioyek"; flake = false; };
    base16-sublime-merge = { url = "github:tinted-theming/base16-sublime-merge"; flake = false; };
    base16-vim = { url = "github:tinted-theming/base16-vim"; flake = false; };
    base16-vivid = { url = "github:tinted-theming/base16-vivid"; flake = false; };
    base16-waybar = { url = "github:tinted-theming/base16-waybar"; flake = false; };
    base16-windows-terminal = { url = "github:tinted-theming/base16-windows-terminal"; flake = false; };
    base16-wob = { url = "github:tinted-theming/base16-wob"; flake = false; };
    base16-zed = { url = "github:tinted-theming/base16-zed"; flake = false; };
    base24-css-etc = { url = "github:tinted-theming/base24-css-etc"; flake = false; };
    base24-gnome-terminal = { url = "github:tinted-theming/base24-gnome-terminal"; flake = false; };
    base24-kate = { url = "github:tinted-theming/base24-kate"; flake = false; };
    base24-kdeplasma = { url = "github:tinted-theming/base24-kdeplasma"; flake = false; };
    base24-konsole = { url = "github:tinted-theming/base24-konsole"; flake = false; };
    base24-slack = { url = "github:tinted-theming/base24-slack"; flake = false; };
    base24-termux = { url = "github:tinted-theming/base24-termux"; flake = false; };
    base24-vscode-terminal = { url = "github:tinted-theming/base24-vscode-terminal"; flake = false; };
    base24-windows-terminal = { url = "github:tinted-theming/base24-windows-terminal"; flake = false; };
    base24-xfce4-terminal = { url = "github:tinted-theming/base24-xfce4-terminal"; flake = false; };
  };

  outputs = { self, nixpkgs, schemes
    , base16-dunst, base16-emacs, base16-gtk-flatcolor, base16-helix, base16-hexchat
    , base16-i3, base16-jetbrains, base16-kakoune, base16-kdeplasma, base16-polybar
    , base16-qutebrowser, base16-rofi, base16-scintillua, base16-sioyek, base16-sublime-merge
    , base16-vim, base16-vivid, base16-waybar, base16-windows-terminal, base16-wob, base16-zed
    , base24-css-etc, base24-gnome-terminal, base24-kate, base24-kdeplasma, base24-konsole
    , base24-slack, base24-termux, base24-vscode-terminal, base24-windows-terminal, base24-xfce4-terminal
    }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      templateLinks = dir: ''
        ln -s ${base16-dunst} ${dir}/base16-dunst
        ln -s ${base16-emacs} ${dir}/base16-emacs
        ln -s ${base16-gtk-flatcolor} ${dir}/base16-gtk-flatcolor
        ln -s ${base16-helix} ${dir}/base16-helix
        ln -s ${base16-hexchat} ${dir}/base16-hexchat
        ln -s ${base16-i3} ${dir}/base16-i3
        ln -s ${base16-jetbrains} ${dir}/base16-jetbrains
        ln -s ${base16-kakoune} ${dir}/base16-kakoune
        ln -s ${base16-kdeplasma} ${dir}/base16-kdeplasma
        ln -s ${base16-polybar} ${dir}/base16-polybar
        ln -s ${base16-qutebrowser} ${dir}/base16-qutebrowser
        ln -s ${base16-rofi} ${dir}/base16-rofi
        ln -s ${base16-scintillua} ${dir}/base16-scintillua
        ln -s ${base16-sioyek} ${dir}/base16-sioyek
        ln -s ${base16-sublime-merge} ${dir}/base16-sublime-merge
        ln -s ${base16-vim} ${dir}/base16-vim
        ln -s ${base16-vivid} ${dir}/base16-vivid
        ln -s ${base16-waybar} ${dir}/base16-waybar
        ln -s ${base16-windows-terminal} ${dir}/base16-windows-terminal
        ln -s ${base16-wob} ${dir}/base16-wob
        ln -s ${base16-zed} ${dir}/base16-zed
        ln -s ${base24-css-etc} ${dir}/base24-css-etc
        ln -s ${base24-gnome-terminal} ${dir}/base24-gnome-terminal
        ln -s ${base24-kate} ${dir}/base24-kate
        ln -s ${base24-kdeplasma} ${dir}/base24-kdeplasma
        ln -s ${base24-konsole} ${dir}/base24-konsole
        ln -s ${base24-slack} ${dir}/base24-slack
        ln -s ${base24-termux} ${dir}/base24-termux
        ln -s ${base24-vscode-terminal} ${dir}/base24-vscode-terminal
        ln -s ${base24-windows-terminal} ${dir}/base24-windows-terminal
        ln -s ${base24-xfce4-terminal} ${dir}/base24-xfce4-terminal
      '';
    in
    {
      packages = forAllSystems (pkgs: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "base16-server";
          version = "0.1.0";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;

          postPatch = ''
            mkdir -p data/templates
            ln -s ${schemes} data/schemes
            ${templateLinks "data/templates"}
          '';

          postInstall = ''
            mkdir -p $out/share/base16-server
            cp -r data $out/share/base16-server/
            cp -r templates $out/share/base16-server/
          '';
        };
      });

      nixosModules.default = { pkgs, config, lib, ... }:
        let cfg = config.services.base16-server; in
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
