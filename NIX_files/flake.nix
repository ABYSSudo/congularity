{
  description = "My NixOS for testing";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  nix.settings.experimental-features = ["nix-command flakes"];
  outputs = { self, nixpkgs }: {
    nixosConfigurations.my-nixos = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./packages.nix
    };


  };
}
