{ pkgs, lib, ... }:

{
  packages =
    [
      pkgs.cargo-binstall
      pkgs.cargo-run-bin
      pkgs.dprint
      pkgs.gel
      pkgs.nixfmt-rfc-style
      pkgs.rustup
      pkgs.shfmt
    ]
    ++ lib.optionals pkgs.stdenv.isDarwin [
      pkgs.libiconv
      pkgs.coreutils
    ];

  # disable dotenv since it breaks the variable interpolation supported by `direnv`
  dotenv.disableHint = true;

  scripts."install:all" = {
    exec = ''
      set -e
      install:cargo:bin
    '';
    description = "Install all dependencies.";
  };
  scripts."gelx" = {
    exec = ''
      set -e
      cargo run --package gelx_cli --bin gelx -- $@
    '';
    description = "Install all dependencies.";
  };
  scripts."install:cargo:bin" = {
    exec = ''
      cargo bin --install
    '';
    description = "Install cargo binaries locally.";
  };
  scripts."db:destroy" = {
    exec = ''
      set -e
      gel instance destroy -I $GEL_INSTANCE --non-interactive --force
    '';
    description = "Destroy the local database.";
  };
  scripts."db:setup" = {
    exec = ''
      set -e

      if [ ! -f "$DEVENV_ROOT/.env" ]; then
        cp $DEVENV_ROOT/.env.example $DEVENV_ROOT/.env
        export $(cat .env | xargs)
      fi

      gel instance create --non-interactive $GEL_INSTANCE $GEL_BRANCH || true
      gel instance start --instance $GEL_INSTANCE
      gel migrate
    '';
    description = "Setup the local database.";
  };
  scripts."db:up" = {
    exec = ''
      set -e
      gel watch --instance $GEL_INSTANCE --migrate
    '';
    description = "Watch changes to the local database.";
  };
  scripts."update:deps" = {
    exec = ''
      set -e
      cargo update
      devenv update
    '';
    description = "Update all project dependencies.";
  };
  scripts."build:all" = {
    exec = ''
      cargo build --all-features
    '';
    description = "Build all crates with all features activated.";
  };
  scripts."build:docs" = {
    exec = ''
      RUSTUP_TOOLCHAIN="nightly" cargo doc --all-features --workspace
    '';
    description = "Build documentation site.";
  };
  scripts."fix:all" = {
    exec = ''
      set -e
      fix:clippy
      fix:format
      fix:gelx
      cargo deny check
    '';
    description = "Fix all fixable lint issues.";
  };
  scripts."fix:format" = {
    exec = ''
      set -e
      dprint fmt --config "$DEVENV_ROOT/dprint.json"
    '';
    description = "Fix formatting for entire project.";
  };
  scripts."fix:clippy" = {
    exec = ''
      set -e
      cargo clippy --fix --allow-dirty --allow-staged --all-features
    '';
    description = "Fix fixable lint issues raised by rust clippy.";
  };
  scripts."fix:gelx" = {
    exec = ''
      set -e
      cd examples/gelx_example
      gelx generate
    '';
    description = "Fix fixable lint issues raised by gelx.";
  };
  scripts."lint:all" = {
    exec = ''
      set -e
      lint:clippy
      lint:format
      lint:gelx
      cargo deny check
    '';
    description = "Lint all project files.";
  };
  scripts."lint:format" = {
    exec = ''
      set -e
      dprint check
    '';
    description = "Check all formatting is correct.";
  };
  scripts."lint:clippy" = {
    exec = ''
      set -e
      cargo clippy --all-features
    '';
    description = "Check rust clippy lints.";
  };
  scripts."lint:gelx" = {
    exec = ''
      set -e
      cd examples/gelx_example
      gelx check
    '';
    description = "Check gelx is formatted correctly.";
  };
  scripts."test:all" = {
    exec = ''
      set -e
      cargo test_all_features
      cargo test_docs
    '';
    description = "Test all project files.";
  };
  scripts."coverage:all" = {
    exec = ''
      set -e
      cargo coverage_all_features
      cargo coverage_docs
      cargo coverage_report
    '';
    description = "Test all files and generate a coverage report for upload to codecov.";
  };
  scripts."setup:vscode" = {
    exec = ''
      set -e
      rm -rf .vscode
      cp -r $DEVENV_ROOT/setup/editors/vscode .vscode
    '';
    description = "Setup the vscode editor for development.";
  };
  scripts."setup:helix" = {
    exec = ''
      set -e
      rm -rf .helix
      cp -r $DEVENV_ROOT/setup/editors/helix .helix
    '';
    description = "Setup the helix editor for development.";
  };
  scripts."setup:ci" = {
    exec = ''
      set -e
      # update github ci path
      echo "$DEVENV_PROFILE/bin" >> $GITHUB_PATH

      # update github ci environment
      echo "DEVENV_PROFILE=$DEVENV_PROFILE" >> $GITHUB_ENV

      # prepend common compilation lookup paths
      echo "PKG_CONFIG_PATH=$PKG_CONFIG_PATH" >> $GITHUB_ENV
      echo "LD_LIBRARY_PATH=$LD_LIBRARY_PATH" >> $GITHUB_ENV
      echo "LIBRARY_PATH=$LIBRARY_PATH" >> $GITHUB_ENV
      echo "C_INCLUDE_PATH=$C_INCLUDE_PATH" >> $GITHUB_ENV

      # these provide shell completions / default config options
      echo "XDG_DATA_DIRS=$XDG_DATA_DIRS" >> $GITHUB_ENV
      echo "XDG_CONFIG_DIRS=$XDG_CONFIG_DIRS" >> $GITHUB_ENV

      echo "DEVENV_DOTFILE=$DEVENV_DOTFILE" >> $GITHUB_ENV
      echo "DEVENV_PROFILE=$DEVENV_PROFILE" >> $GITHUB_ENV
      echo "DEVENV_ROOT=$DEVENV_ROOT" >> $GITHUB_ENV
      echo "DEVENV_STATE=$DEVENV_STATE" >> $GITHUB_ENV
    '';
    description = "Setup the github ci environment.";
  };
}
