{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    buildInputs = with pkgs; [
      rustc
      cargo
      pkg-config
      openssl
      postgresql_16
    ];

    RUST_BACKTRACE = 1;
}
