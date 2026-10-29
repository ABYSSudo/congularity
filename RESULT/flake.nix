{
  description = "My NixOS for testing via Congularity";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixos-generators, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};

      modules = [
        ./packages.nix
        ./kernel.nix
        ./users.nix
        ./common.nix
      ];

      mkIso = nixos-generators.nixosGenerate {
        inherit system modules;
        format = "iso";
      };

      mkVm = nixos-generators.nixosGenerate {
        inherit system modules;
        format = "vm";
      };

      mkSystem = nixpkgs.lib.nixosSystem {
        inherit system modules;
      };
    in {
      packages.${system} = {
        iso = mkIso;
        vm  = mkVm;
        
        congularity = pkgs.rustPlatform.buildRustPackage {
          pname = "congularity";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ pkgs.pkg-config ];
          
          buildInputs = [ 
            pkgs.gtk4 
            pkgs.gdk-pixbuf 
          ];
        default = self.packages.${system}.congularity;
      };

      nixosConfigurations.my-nixos = mkSystem;
    };
}
