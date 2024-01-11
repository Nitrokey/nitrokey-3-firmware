with import <nixpkgs> {};

stdenv.mkDerivation rec {
  pname = "nrf-command-line-tools";
  version = "10.23.4";

  src = fetchurl {
    url = "https://nsscprodmedia.blob.core.windows.net/prod/software-and-other-downloads/desktop-software/nrf-command-line-tools/sw/versions-10-x-x/10-23-4/nrf-command-line-tools-10.23.4_linux-amd64.tar.gz";
    hash = "sha256-+kastgV9jfvnpGE48O3FSqC93cCL93W2OCyaHV+96+U=";
  };

  nativeBuildInputs = [
    autoPatchelfHook
  ];

  buildInputs = [
    pkgs.libusb1
  ];

  propagatedBuildInputs = [
    pkgs.segger-jlink
  ];

  installPhase = ''
    runHook preInstall
    install -Dm0755 -D bin/* -t $out/bin
    install -Dm0644 include/*.h -t $out/include
    install -Dm0644 lib/*.so -t $out/lib
    cp -rv share $out/share
    runHook postInstall
  '';

  postFixup = ''
    patchelf $out/bin/* --add-rpath ${pkgs.segger-jlink}/lib
  '';

  meta = with lib; {
    homepage = "https://www.nordicsemi.com/Products/Development-tools/nRF-Command-Line-Tools";
    description = "nRF Command Line Tools";
    platforms = platforms.linux;
  };
}
