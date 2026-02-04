{
  description = "Base16/Base24 color scheme server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    schemes = { url = "github:tinted-theming/schemes"; flake = false; };
    base16-btop = { url = "sourcehut:~blueingreen/base16-btop"; flake = false; };
    base16-dunst = { url = "github:tinted-theming/base16-dunst"; flake = false; };
    base16-gnome-terminal = { url = "github:aarowill/base16-gnome-terminal"; flake = false; };
    base16-highlight = { url = "github:bezhermoso/base16-highlight"; flake = false; };
    base16-emacs = { url = "github:tinted-theming/base16-emacs"; flake = false; };
    base16-gtk-flatcolor = { url = "github:tinted-theming/base16-gtk-flatcolor"; flake = false; };
    base16-helix = { url = "github:tinted-theming/base16-helix"; flake = false; };
    base16-hexchat = { url = "github:tinted-theming/base16-hexchat"; flake = false; };
    base16-i3 = { url = "github:tinted-theming/base16-i3"; flake = false; };
    base16-i3status-rust = { url = "github:mystfox/base16-i3status-rust"; flake = false; };
    base16-jetbrains = { url = "github:tinted-theming/base16-jetbrains"; flake = false; };
    base16-kakoune = { url = "github:tinted-theming/base16-kakoune"; flake = false; };
    base16-kdeplasma = { url = "github:tinted-theming/base16-kdeplasma"; flake = false; };
    base16-lazygit = { url = "sourcehut:~blueingreen/base16-lazygit"; flake = false; };
    base16-mako = { url = "github:Eluminae/base16-mako"; flake = false; };
    base16-micro = { url = "github:w3dg/base16-micro"; flake = false; };
    base16-polybar = { url = "github:tinted-theming/base16-polybar"; flake = false; };
    base16-pygments = { url = "github:mohd-akram/base16-pygments"; flake = false; };
    base16-qutebrowser = { url = "github:tinted-theming/base16-qutebrowser"; flake = false; };
    base16-rofi = { url = "github:tinted-theming/base16-rofi"; flake = false; };
    base16-scintillua = { url = "github:tinted-theming/base16-scintillua"; flake = false; };
    base16-sioyek = { url = "github:tinted-theming/base16-sioyek"; flake = false; };
    base16-styles = { url = "github:samme/base16-styles"; flake = false; };
    base16-sublime-merge = { url = "github:tinted-theming/base16-sublime-merge"; flake = false; };
    base16-sway = { url = "github:rkubosz/base16-sway"; flake = false; };
    base16-swaylock = { url = "git+https://git.michaelball.name/gid/base16-swaylock-template"; flake = false; };
    base16-vivid = { url = "github:tinted-theming/base16-vivid"; flake = false; };
    base16-waybar = { url = "github:tinted-theming/base16-waybar"; flake = false; };
    base16-wob = { url = "github:tinted-theming/base16-wob"; flake = false; };
    base16-wofi = { url = "sourcehut:~knezi/base16-wofi"; flake = false; };
    base16-zathura = { url = "github:HaoZeke/base16-zathura"; flake = false; };
    base16-zed = { url = "github:tinted-theming/base16-zed"; flake = false; };
    base24-css-etc = { url = "github:tinted-theming/base24-css-etc"; flake = false; };
    base24-kate = { url = "github:tinted-theming/base24-kate"; flake = false; };
    base24-kdeplasma = { url = "github:tinted-theming/base24-kdeplasma"; flake = false; };
    base24-slack = { url = "github:tinted-theming/base24-slack"; flake = false; };
    tinted-delta = { url = "github:tinted-theming/tinted-delta"; flake = false; };
    tinted-fzf = { url = "github:tinted-theming/tinted-fzf"; flake = false; };
    tinted-jqp = { url = "github:tinted-theming/tinted-jqp"; flake = false; };
    tinted-matlab = { url = "github:tinted-theming/tinted-matlab"; flake = false; };
    tinted-nvim = { url = "github:tinted-theming/tinted-nvim"; flake = false; };
    tinted-shell = { url = "github:tinted-theming/tinted-shell"; flake = false; };
    tinted-sublime-text = { url = "github:tinted-theming/tinted-sublime-text"; flake = false; };
    tinted-terminal = { url = "github:tinted-theming/tinted-terminal"; flake = false; };
    tinted-tmux = { url = "github:tinted-theming/tinted-tmux"; flake = false; };
    tinted-tridactyl = { url = "github:tinted-theming/tinted-tridactyl"; flake = false; };
    tinted-vim = { url = "github:tinted-theming/tinted-vim"; flake = false; };
    tinted-vscode = { url = "github:tinted-theming/tinted-vscode"; flake = false; };
    tinted-xresources = { url = "github:tinted-theming/tinted-xresources"; flake = false; };
    tinted-yazi = { url = "github:tinted-theming/tinted-yazi"; flake = false; };
  };

  outputs = { self, nixpkgs, schemes
    , base16-btop, base16-dunst, base16-emacs, base16-gnome-terminal, base16-gtk-flatcolor
    , base16-helix, base16-hexchat, base16-highlight, base16-i3, base16-i3status-rust
    , base16-jetbrains, base16-kakoune, base16-kdeplasma, base16-lazygit, base16-mako
    , base16-micro, base16-polybar, base16-pygments, base16-qutebrowser, base16-rofi
    , base16-scintillua, base16-sioyek, base16-styles, base16-sublime-merge, base16-sway
    , base16-swaylock, base16-vivid, base16-waybar, base16-wob, base16-wofi, base16-zathura
    , base16-zed
    , base24-css-etc, base24-kate, base24-kdeplasma, base24-slack
    , tinted-delta, tinted-fzf, tinted-jqp, tinted-matlab, tinted-nvim, tinted-shell
    , tinted-sublime-text, tinted-terminal, tinted-tmux, tinted-tridactyl
    , tinted-vim, tinted-vscode, tinted-xresources, tinted-yazi
    }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      templateLinks = dir: ''
        ln -s ${base16-btop} ${dir}/base16-btop
        ln -s ${base16-dunst} ${dir}/base16-dunst
        ln -s ${base16-emacs} ${dir}/base16-emacs
        ln -s ${base16-gnome-terminal} ${dir}/base16-gnome-terminal
        ln -s ${base16-gtk-flatcolor} ${dir}/base16-gtk-flatcolor
        ln -s ${base16-helix} ${dir}/base16-helix
        ln -s ${base16-hexchat} ${dir}/base16-hexchat
        ln -s ${base16-highlight} ${dir}/base16-highlight
        ln -s ${base16-i3} ${dir}/base16-i3
        ln -s ${base16-i3status-rust} ${dir}/base16-i3status-rust
        ln -s ${base16-jetbrains} ${dir}/base16-jetbrains
        ln -s ${base16-kakoune} ${dir}/base16-kakoune
        ln -s ${base16-kdeplasma} ${dir}/base16-kdeplasma
        ln -s ${base16-lazygit} ${dir}/base16-lazygit
        ln -s ${base16-mako} ${dir}/base16-mako
        ln -s ${base16-micro} ${dir}/base16-micro
        ln -s ${base16-polybar} ${dir}/base16-polybar
        ln -s ${base16-pygments} ${dir}/base16-pygments
        ln -s ${base16-qutebrowser} ${dir}/base16-qutebrowser
        ln -s ${base16-rofi} ${dir}/base16-rofi
        ln -s ${base16-scintillua} ${dir}/base16-scintillua
        ln -s ${base16-sioyek} ${dir}/base16-sioyek
        ln -s ${base16-styles} ${dir}/base16-styles
        ln -s ${base16-sublime-merge} ${dir}/base16-sublime-merge
        ln -s ${base16-sway} ${dir}/base16-sway
        ln -s ${base16-swaylock} ${dir}/base16-swaylock
        ln -s ${base16-vivid} ${dir}/base16-vivid
        ln -s ${base16-waybar} ${dir}/base16-waybar
        ln -s ${base16-wob} ${dir}/base16-wob
        ln -s ${base16-wofi} ${dir}/base16-wofi
        ln -s ${base16-zathura} ${dir}/base16-zathura
        ln -s ${base16-zed} ${dir}/base16-zed
        ln -s ${base24-css-etc} ${dir}/base24-css-etc
        ln -s ${base24-kate} ${dir}/base24-kate
        ln -s ${base24-kdeplasma} ${dir}/base24-kdeplasma
        ln -s ${base24-slack} ${dir}/base24-slack
        ln -s ${tinted-delta} ${dir}/tinted-delta
        ln -s ${tinted-fzf} ${dir}/tinted-fzf
        ln -s ${tinted-jqp} ${dir}/tinted-jqp
        ln -s ${tinted-matlab} ${dir}/tinted-matlab
        ln -s ${tinted-nvim} ${dir}/tinted-nvim
        ln -s ${tinted-shell} ${dir}/tinted-shell
        ln -s ${tinted-sublime-text} ${dir}/tinted-sublime-text
        ln -s ${tinted-terminal} ${dir}/tinted-terminal
        ln -s ${tinted-tmux} ${dir}/tinted-tmux
        ln -s ${tinted-tridactyl} ${dir}/tinted-tridactyl
        ln -s ${tinted-vim} ${dir}/tinted-vim
        ln -s ${tinted-vscode} ${dir}/tinted-vscode
        ln -s ${tinted-xresources} ${dir}/tinted-xresources
        ln -s ${tinted-yazi} ${dir}/tinted-yazi
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
            cp -r .cache $out/share/base16-server/
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
