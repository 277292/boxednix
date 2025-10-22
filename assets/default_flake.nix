{
  outputs = {nixpkgs, ...}: let
    inherit (builtins) readDir elem;
    inherit (nixpkgs.lib) concatMapAttrs mapAttrs' nameValuePair getAttr filterAttrs flip;

    match = flip getAttr;

    read_dir_recursively = dir: let
      ignore_dirs = [".git" ".direnv" "flake.nix" "flake.lock"];
    in
      concatMapAttrs (
        this:
          match {
            directory = mapAttrs' (subpath: nameValuePair "${this}/${subpath}") (read_dir_recursively "${dir}/${this}");
            regular = {${this} = "${dir}/${this}";};
            symlink = {};
          }
      ) (filterAttrs (name: _value: !(elem name ignore_dirs)) (readDir dir));
  in {
    files = read_dir_recursively ./.;
  };
}
