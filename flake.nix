{
  inputs.nixpkgs.url = "nixpkgs";

  outputs = {
    self,
    nixpkgs,
    ...
  }: let
    version = builtins.substring 0 7 self.lastModifiedDate;

    systems = [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];

    forAllSystems = nixpkgs.lib.genAttrs systems;
    nixpkgsFor = forAllSystems (system: import nixpkgs {inherit system;});
  in rec {
    devShells = forAllSystems (s: let
      pkgs = nixpkgsFor.${s};
      inherit (pkgs) mkShell;
      libs = with pkgs; [
        glib
        gtk4
      ];
    in {
      default = mkShell {
        packages = with pkgs; [pkg-config] ++ libs;
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libs;
      };
    });
  };
}