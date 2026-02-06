{
  inputs = { nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable"; };

  outputs = { self, nixpkgs }:
    let pkgs = nixpkgs.legacyPackages."x86_64-linux";

    in
    {
      devShells."x86_64-linux".default = pkgs.mkShell {

        buildInputs = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
          rust-analyzer
          glib
          rustup
          # The Shell
          nushell
        ];
        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        SHELL = "${pkgs.nushell}/bin/nu";
        shellHook = ''
                    export SHELL=${pkgs.nushell}/bin/nu
          				'';

      };
    };

}
