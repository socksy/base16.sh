#!/bin/bash
set -e

TEMPLATE_DIR="data/templates"
mkdir -p "$TEMPLATE_DIR"

REPOS=(
    "base16-dunst"
    "base16-emacs"
    "base16-gtk-flatcolor"
    "base16-helix"
    "base16-hexchat"
    "base16-i3"
    "base16-jetbrains"
    "base16-kakoune"
    "base16-kdeplasma"
    "base16-polybar"
    "base16-qutebrowser"
    "base16-rofi"
    "base16-scintillua"
    "base16-sioyek"
    "base16-sublime-merge"
    "base16-vim"
    "base16-vivid"
    "base16-waybar"
    "base16-windows-terminal"
    "base16-wob"
    "base16-zed"
    "base24-css-etc"
    "base24-gnome-terminal"
    "base24-kate"
    "base24-kdeplasma"
    "base24-konsole"
    "base24-slack"
    "base24-termux"
    "base24-vscode-terminal"
    "base24-windows-terminal"
    "base24-xfce4-terminal"
)

for repo in "${REPOS[@]}"; do
    echo "Cloning $repo..."
    git clone --depth 1 "https://github.com/tinted-theming/$repo.git" "$TEMPLATE_DIR/$repo" 2>&1 | grep -v "Receiving objects" || true
done

echo "Done! Cloned ${#REPOS[@]} template repositories."
