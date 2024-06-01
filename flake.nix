{
  description = "A flake for git-ce, with Hercules CI support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs@{ self, flake-parts, nixpkgs, crane, flake-utils, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-darwin" ];
      perSystem = { pkgs, system, ... }:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
          craneLib = crane.mkLib pkgs;
          commonArgs = {
            src = craneLib.cleanCargoSource (craneLib.path ./.);
            buildInputs = with pkgs; [
              openssl
              pkg-config
              libgit2
              # Add additional build inputs here
            ] ++ lib.optionals stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              libiconv
            ];

          };
          cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
            pname = "git-ce-deps";
          });

          git-ce-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          git-ce-nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
          });


          git-ce = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
        in
        {
          checks = {
            inherit
              git-ce
              git-ce-clippy
              git-ce-nextest;
          };
          packages.default = git-ce;

          apps.default = flake-utils.lib.mkApp {
            drv = git-ce;
          };



          devShells.default = pkgs.mkShell {

            # Additional dev-shell environment variables can be set directly
            # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

            # Extra inputs can be added here
            nativeBuildInputs = with pkgs; [
              cargo
              rustc
              rust-analyzer
              pkg-config
              openssl
              libgit2
              git-cliff
            ];
          };
        };

    };

  nixConfig = {
    extra-substituters = [ "https://git-ce.cachix.org" ];
    extra-trusted-public-keys = [ "git-ce.cachix.org-1:U+Gm5iuIbU4Q/RKIlK1eCB5HPXH5eqDTlp4tbOjG30M=" ];
  };
}
