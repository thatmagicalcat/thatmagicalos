{
    pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
    buildInputs = with pkgs; [
        libclang
    ];

    LIBCLANG_PATH="${pkgs.libclang.lib}/lib";
}
