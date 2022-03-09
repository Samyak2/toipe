with import <nixpkgs> {};
mkShell {
  packages = [
    rustc
    cargo
    clippy
    rust-analyzer
    rustfmt
  ];
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  shellHook = ''
    alias run="cargo run"
    alias build="cargo build"
  '';
}
