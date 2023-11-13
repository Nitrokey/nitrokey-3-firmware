# Commit from 2023-11-10
{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/da4024d0ead5d7820f6bd15147d3fe2a0c0cec73.tar.gz") {} }:

let
  fenix = pkgs.callPackage
    (pkgs.fetchFromGitHub {
      owner = "nix-community";
      repo = "fenix";
      # commit from: 2023-11-01
      rev = "1a92c6d75963fd594116913c23041da48ed9e020";
      hash = "sha256-L3vZfifHmog7sJvzXk8qiKISkpyltb+GaThqMJ7PU9Y=";
    })
    { };
  toolchain = fenix.fromToolchainFile {
    dir = ../..;
    sha256 = "sha256-Q9UgzzvxLi4x9aWUJTn+/5EXekC98ODRU1TwhUs9RnY=";
  };
in
pkgs.mkShell {
  name = "nk3";
  nativeBuildInputs = with pkgs.buildPackages; [
    flip-link
    gcc-arm-embedded
    git
    gnumake
    libclang
    toolchain
    (python3.withPackages(ps: with ps; [ toml ]))
  ];

  shellHook = ''
    export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
    export TARGET_CC=arm-none-eabi-gcc
  '';
}
