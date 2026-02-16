#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <serve|build> [extra args]" >&2
  exit 2
fi

mode="$1"
shift || true

# Tauri may forward CLI styling flags that Trunk does not parse the same way.
while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-color)
      if [[ $# -ge 2 ]]; then
        shift 2
      else
        shift 1
      fi
      ;;
    --color)
      if [[ $# -ge 2 ]]; then
        shift 2
      else
        shift 1
      fi
      ;;
    *)
      shift 1
      ;;
  esac
done

case "$mode" in
  serve)
    unset NO_COLOR || true
    export TRUNK_NO_COLOR=false
    exec trunk serve --config ui/Trunk.toml
    ;;
  build)
    unset NO_COLOR || true
    export TRUNK_NO_COLOR=false
    exec trunk build --config ui/Trunk.toml
    ;;
  *)
    echo "unknown mode: $mode" >&2
    exit 2
    ;;
esac
