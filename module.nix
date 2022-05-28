{ self }:

{ config, pkgs, lib, ... }:

with lib;
let
  cfg = config.services.label-tracker;
in {
  options = {
    services.label-tracker = {
      enable = mkEnableOption "the github label tracer";

      apiToken = mkOption {
        type = types.path;
        description = ''
          Path to the github api token file. Must follow the format
          of systemd EnvironmentFile and contain a GITHUB_API_TOKEN
          with an appropriate token.
        '';
      };

      group = mkOption {
        type = types.str;
        description = ''
          Group to run with. Should usually be the webserver group.
        '';
      };

      startAt = mkOption {
        type = types.either types.str (types.listOf types.str);
        default = [];
        description = ''
          When to run syncs for all tracked labels. See systemd.time(7).
        '';
      };

      track = mkOption {
        type = types.attrsOf (types.submodule {
          options = {
            owner = mkOption {
              type = types.str;
              description = "Owner of the repo.";
            };

            repo = mkOption {
              type = types.str;
              description = "Name of the repo.";
            };

            label = mkOption {
              type = types.str;
              description = "Name of the label.";
            };
          };
        });
        default = [];
        description = ''
          Repos and labels to track.
        '';
      };

      feedAgeLimit = mkOption {
        type = types.ints.unsigned;
        default = 240;
        description = ''
          Age cutoff for generated feed entries, in hours.
        '';
      };
    };
  };

  config = mkIf cfg.enable {
    users.users.label-tracker = {
      isSystemUser = true;
      group = cfg.group;
    };

    systemd.services.label-tracker = {
      startAt = cfg.startAt;

      path = [ self.packages.${config.nixpkgs.system}.label-tracker ];
      environment.RUST_LOG = "info";
      script = ''
        set -euo pipefail
        shopt -s inherit_errexit failglob

        ${concatStringsSep "\n"
          (mapAttrsToList
            (key: args:
              let
                name = escapeShellArg key;
                owner = escapeShellArg args.owner;
                repo = escapeShellArg args.repo;
                label = escapeShellArg args.label;
              in ''
                (
                  umask 0077
                  if ! [ -e states/${name} ]; then
                    mkdir -p states
                    label-tracker init states/${name} ${owner} ${repo} ${label}
                  fi
                  label-tracker sync-issues states/${name}
                  label-tracker sync-prs states/${name}
                )
                (
                  umask 0027
                  mkdir -p results/${name}
                  umask 0037
                  label-tracker emit-issues states/${name} \
                    -a ${toString cfg.feedAgeLimit} \
                    > results/${name}/.issues.xml.tmp
                  mv results/${name}/.issues.xml.tmp results/${name}/issues.xml
                  label-tracker emit-prs states/${name} \
                    -a ${toString cfg.feedAgeLimit} \
                    > results/${name}/.prs.xml.tmp
                  mv results/${name}/.prs.xml.tmp results/${name}/prs.xml
                )
              '')
            cfg.track)}
      '';

      serviceConfig = {
        EnvironmentFile = cfg.apiToken;

        User = "label-tracker";
        Group = cfg.group;
        StateDirectory = "label-tracker";
        WorkingDirectory = "/var/lib/label-tracker";
        AmbientCapabilities = [ "" ];
        CapabilityBoundingSet = [ "" ];
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateTmp = true;
        ProcSubset = "pid";
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectProc = "invisible";
        ProtectSystem = true;
        RemoveIPC = true;
        RestrictAddressFamilies = "AF_INET AF_INET6";
        RestrictNamespaces = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = [
          "@system-service"
          "~ @resources @privileged"
        ];
        UMask = "0027";
      };
    };
  };
}
