self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs."spider";
in
{
  options.programs."spider" = {
    enable = lib.mkEnableOption "Multi-screen info panel (Over ESP32).";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "spider.packages.\${system}.default";
      description = "Multi-screen info panel (Over ESP32).";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
  };
}
