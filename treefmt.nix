{
  projectRootFile = "flake.nix";
  programs = {
    alejandra.enable = true;
    jsonfmt.enable = true;
    mdformat.enable = true;
    ruff-check.enable = true;
    ruff-format.enable = true;
    shfmt.enable = true;
    toml-sort.enable = true;
    yamlfmt.enable = true;
  };
  settings = {
    on-unmatched = "info";
    excludes = [
      "*.conf"
      "*.css"
      "*.pub"
      "flake.lock"
      "*.ini"
    ];
  };
  settings.formatter.shfmt = {
    includes = [
      "*.sh"
    ];
  };
}
