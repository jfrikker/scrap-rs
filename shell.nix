{ pkgs ? import <nixpkgs> {} }:
  pkgs.clangStdenv.mkDerivation {
    name = "scrap-dev";
    src = null;
    # nativeBuildInputs is usually what you want -- tools you need to run
    # nativeBuildInputs = [ pkgs.clang ];
    buildInputs = [
      pkgs.libffi
      pkgs.libxml2
      pkgs.llvmPackages_14.llvm
    ];
    shellHook = ''
      export LLVM_SYS_140_PREFIX="${pkgs.llvmPackages_14.llvm.dev}";
    '';
}
