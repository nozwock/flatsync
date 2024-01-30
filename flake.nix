{
  description = "Flatsync";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;  
    in
    {
      devShells.x86_64-linux.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          
          git
          pkg-config
          cmake
          glib
          gtk4
          ninja
          python311Packages.pygobject3
          libadwaita
          flatpak
          gobject-introspection
          desktop-file-utils
        ];
      };
    };
}
