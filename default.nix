{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    rustup
    openssl
    pkg-config
    pandoc
  ];

  shellHook = '''';
}
