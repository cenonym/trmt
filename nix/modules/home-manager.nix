self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.programs.trmt;
  settingsFormat = pkgs.formats.toml {};
in {
  options.programs.trmt = {
    enable = lib.mkEnableOption "trmt; 2D Turing machine (turmite) for your terminal, written in Rust";

    package = lib.mkPackageOption self.packages.${pkgs.system} "trmt" {
      default = "default";
      pkgsText = "trmt.packages.\${system}";
    };

    config = lib.mkOption {
      type = settingsFormat.type;
      example = lib.literalExpression ''
        {
          simulation = {
            autoplay = true;
            heads = 3;
            rule = "RL";
            speed_ms = 20;
            trail_length = 24;
            color_cells = true;
            seed = "";
          };
          display = {
            keycast = false;
            colors = ["rgb(241, 113, 54)" "#45a8e9" "229"];
            fade_trail_color = "";
            state_based_colors = false;
            live_colors = false;
            head_char = ["██"];
            trail_char = ["▓▓"];
            cell_char = "░░";
            randomize_heads = false;
            randomize_trails = false;
            direction_based_chars = false
          };
          controls = {
            quit = "q";
            toggle = " ";
            reset = "r";
            faster = "+";
            slower = "-";
            config_reload = "c";
            help = "h";
            statusbar = "b";
            randomize_seed = "s";
            randomize_rule = "n";
            randomize = "R";
          };
        };
      '';
      description = ''
        trmt configuration.
        For available settings, see <https://github.com/cenonym/trmt?tab=readme-ov-file#config-options>.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = lib.mkIf (cfg.package != null) [cfg.package];

    xdg.configFile."trmt/config.toml".source = settingsFormat.generate "config.toml" cfg.config;
  };
}
