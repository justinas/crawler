{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  buildInputs = with pkgs; [ cargo cargo-watch openssl pkg-config rustc ];
}
