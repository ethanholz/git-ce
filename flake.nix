{
  description = "git-ce - Git conventional commits tools";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };
  outputs = {
    self,
    nixpkgs,
  }: let
    makePackage = system: releaseInfo: let
      pkgs = nixpkgs.legacyPackages.${system};
    in
      pkgs.stdenvNoCC.mkDerivation {
        pname = "git-ce";
        version = "0.5.0";
        src = pkgs.fetchurl releaseInfo;
        dontUnpack = true;
        installPhase = ''
          mkdir -p $out/bin
          cp $src $out/bin/git-ce
          chmod +x $out/bin/git-ce
        '';
      };

    releases = {
      x86_64-linux = {
        url = "https://github.com/ethanholz/git-ce/releases/download/v0.5.0/git-ce-x86_64-unknown-linux-musl-0.5.0";
        sha256 = "1y14maxmfacb3kzvb72xr3l4rvvwzvji5y5vrx91mbhzmx3r6jph";
      };
      aarch64-darwin = {
        url = "https://github.com/ethanholz/git-ce/releases/download/v0.5.0/git-ce-aarch64-apple-darwin-0.5.0";
        sha256 = "0aaay03z15dwkhhmyxjlk2y7471fz281srhfsz68wjdfa8agk3ff";
      };
      aarch64-linux = {
        url = "https://github.com/ethanholz/git-ce/releases/download/v0.5.0/git-ce-aarch64-unknown-linux-musl-0.5.0";
        sha256 = "190dm8yg307jz54fq3hgjiiwdsvv6i3m6hzszpv3769amjq8nsg9";
      };
      x86_64-darwin = {
        url = "https://github.com/ethanholz/git-ce/releases/download/v0.5.0/git-ce-x86_64-apple-darwin-0.5.0";
        sha256 = "06ar8r543linhjr3xjc2qk2w6fazizahqjhvi8101fmimdihnwl5";
      };
    };
  in {
    packages =
      builtins.mapAttrs (system: releaseInfo: {
        default = makePackage system releaseInfo;
      })
      releases;
  };
}
