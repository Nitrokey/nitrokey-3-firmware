{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  packages = [ pkgs.uv ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.cairo
  ];
}
