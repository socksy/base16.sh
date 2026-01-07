# base16.sh

Rust server for serving Base16/Base24 color schemes and rendering them with Mustache templates.

## What it does

- `base16.sh/solarized-light` → Returns the YAML scheme
- `base16.sh/solarized-light/vim` → Returns a rendered vim theme

Same for `base24.sh` with Base24 schemes.

## Status

Work in progress. Building a fast in-memory server that fetches schemes from [tinted-theming/schemes](https://github.com/tinted-theming/schemes) and renders them with templates from [base16-templates-source](https://github.com/chriskempson/base16-templates-source).

## Dev

Uses devenv for development setup. Run `devenv shell` to get started.

## License

AGPL-3.0-or-later
