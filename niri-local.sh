#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
binary="$repo_dir/target/release/niri"
personal_config="${XDG_CONFIG_HOME:-$HOME/.config}/niri-personal/config.kdl"

usage() {
    cat <<'EOF'
Usage: niri-local.sh [--nested | --session | --validate]

With no option, open a nested test window from a desktop, or start a full
personal session from a TTY.

  --nested    Test in a window (Alt+Enter opens terminals; Alt+wheel pans).
  --session   Start the personal compositor after logging out, from a TTY.
  --validate  Validate the personal config without starting a compositor.
  --help      Show this help.
EOF
}

if (( $# > 1 )); then
    usage >&2
    exit 2
fi

mode="${1:-auto}"
case "$mode" in
    --help|-h) usage; exit 0 ;;
    auto|--nested|--session|--validate) ;;
    *) usage >&2; exit 2 ;;
esac

if [[ ! -x "$binary" ]]; then
    echo "Build the local binary first: cd '$repo_dir' && cargo build --release" >&2
    exit 1
fi

if [[ "$mode" == auto ]]; then
    if [[ -n "${WAYLAND_DISPLAY:-}${WAYLAND_SOCKET:-}${DISPLAY:-}" ]]; then
        mode=--nested
    else
        mode=--session
    fi
fi

case "$mode" in
    --nested)
        if [[ -z "${WAYLAND_DISPLAY:-}${WAYLAND_SOCKET:-}${DISPLAY:-}" ]]; then
            echo "Nested mode needs an existing desktop session." >&2
            exit 1
        fi
        exec "$binary" --config "$repo_dir/resources/local-test-config.kdl"
        ;;
    --session)
        if [[ -n "${WAYLAND_DISPLAY:-}${WAYLAND_SOCKET:-}${DISPLAY:-}" ]]; then
            echo "Log out and run this from a TTY, or use --nested to test in a window." >&2
            exit 1
        fi
        exec "$binary" --session --config "$personal_config"
        ;;
    --validate)
        exec "$binary" validate --config "$personal_config"
        ;;
esac
