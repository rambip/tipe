{pkgs? import <nixpkgs> {} }:
let 
    sources = import ./nix/sources.nix;
    wasm-tooling= pkgs.callPackage sources.wasm-tooling {};
in
    wasm-tooling.rust.buildWithTrunk {
        src = ./.;
    }
