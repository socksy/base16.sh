{ pkgs, lib, config, inputs, ... }:

{
  packages = with pkgs; [
    git
    openssl
    pkg-config

    # Load testing tools
    wrk
    k6
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    version = "1.91.1";
  };

  env = {
    RUST_BACKTRACE = "1";
    RUST_LOG = "info";
  };

  enterShell = ''
    echo "🎨 Base16/Base24 Theme Server Development Environment"
    echo ""
    echo "Rust toolchain: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo ""
    echo "Available commands:"
    echo "  cargo build          - Build the project"
    echo "  cargo test           - Run tests"
    echo "  cargo clippy         - Run linter"
    echo "  cargo fmt            - Format code"
    echo "  cargo run            - Run the server"
    echo "  cargo audit          - Security audit"
    echo "  wrk / k6             - Load testing tools"
    echo ""
  '';

  processes = {
    # Optional: can run the server as a process
    # server.exec = "cargo run --release";
  };

  scripts = {
    dev.exec = ''
      cargo watch -x 'run' -x test
    '';

    lint.exec = ''
      cargo clippy --all-targets --all-features -- -D warnings
    '';

    fmt-check.exec = ''
      cargo fmt -- --check
    '';

    audit.exec = ''
      cargo audit
    '';
  };
}
