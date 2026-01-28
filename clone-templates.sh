#!/usr/bin/env bash
set -e

TEMPLATE_DIR="data/templates"
mkdir -p "$TEMPLATE_DIR"

REPOS=(
    "sr.ht:~blueingreen/base16-btop"
    "base16-dunst"
    "base16-emacs"
    "aarowill/base16-gnome-terminal"
    "base16-gtk-flatcolor"
    "base16-helix"
    "base16-hexchat"
    "bezhermoso/base16-highlight"
    "base16-i3"
    "mystfox/base16-i3status-rust"
    "base16-jetbrains"
    "base16-kakoune"
    "base16-kdeplasma"
    "sr.ht:~blueingreen/base16-lazygit"
    "Eluminae/base16-mako"
    "w3dg/base16-micro"
    "base16-polybar"
    "mohd-akram/base16-pygments"
    "base16-qutebrowser"
    "base16-rofi"
    "base16-scintillua"
    "base16-sioyek"
    "samme/base16-styles"
    "base16-sublime-merge"
    "rkubosz/base16-sway"
    "git.michaelball.name:gid/base16-swaylock-template"
    "base16-vivid"
    "base16-waybar"
    "base16-wob"
    "sr.ht:~knezi/base16-wofi"
    "HaoZeke/base16-zathura"
    "base16-zed"
    "base24-css-etc"
    "base24-kate"
    "base24-kdeplasma"
    "base24-slack"
    "tinted-delta"
    "tinted-fzf"
    "tinted-jqp"
    "tinted-matlab"
    "tinted-nvim"
    "tinted-shell"
    "tinted-sublime-text"
    "tinted-terminal"
    "tinted-tmux"
    "tinted-tridactyl"
    "tinted-vim"
    "tinted-vscode"
    "tinted-xresources"
    "tinted-yazi"
)

for repo in "${REPOS[@]}"; do
    echo "Cloning $repo..."
    if [[ "$repo" == sr.ht:* ]]; then
        # sr.ht repo like "sr.ht:~blueingreen/base16-btop"
        srht_path="${repo#sr.ht:}"
        git clone --depth 1 "https://git.sr.ht/$srht_path" "$TEMPLATE_DIR/${srht_path##*/}" 2>&1 | grep -v "Receiving objects" || true
    elif [[ "$repo" == git.michaelball.name:* ]]; then
        # Custom git host
        path="${repo#git.michaelball.name:}"
        git clone --depth 1 "https://git.michaelball.name/$path" "$TEMPLATE_DIR/${path##*/}" 2>&1 | grep -v "Receiving objects" || true
    elif [[ "$repo" == */* ]]; then
        # GitHub with org like "samme/base16-styles"
        git clone --depth 1 "https://github.com/$repo.git" "$TEMPLATE_DIR/${repo##*/}" 2>&1 | grep -v "Receiving objects" || true
    else
        # Default: tinted-theming org on GitHub
        git clone --depth 1 "https://github.com/tinted-theming/$repo.git" "$TEMPLATE_DIR/$repo" 2>&1 | grep -v "Receiving objects" || true
    fi
done

echo "Done! Cloned ${#REPOS[@]} template repositories."
