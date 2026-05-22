#!/usr/bin/env bash
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

binary_name="unified-model-proxy-v2"
release_binary="$repo_root/target/release/$binary_name"
installed_binary="$repo_root/bin/$binary_name"
repo_plist="$repo_root/launchd/dev.unified-model-proxy-v2.plist"
user_plist="$HOME/Library/LaunchAgents/dev.unified-model-proxy-v2.plist"
launchd_label="dev.unified-model-proxy-v2"
launchd_domain="gui/$(id -u)"
launchd_target="$launchd_domain/$launchd_label"
health_url="http://127.0.0.1:18743/health"

echo "pre-commit: building release binary"
cargo build --release

echo "pre-commit: installing release binary to $installed_binary"
mkdir -p "$repo_root/bin"
install -m 755 "$release_binary" "$installed_binary"

echo "pre-commit: syncing launchd plist to $user_plist"
mkdir -p "$(dirname "$user_plist")"
cp "$repo_plist" "$user_plist"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "pre-commit: non-Darwin host, skipping launchd reload"
  exit 0
fi

if ! command -v launchctl >/dev/null 2>&1; then
  echo "pre-commit: launchctl not available, skipping launchd reload"
  exit 0
fi

echo "pre-commit: reloading launchd service $launchd_target"
launchctl bootout "$launchd_target" >/dev/null 2>&1 || true

for _attempt in $(seq 1 20); do
  if ! launchctl print "$launchd_target" >/dev/null 2>&1; then
    break
  fi
  sleep 0.25
done

if launchctl print "$launchd_target" >/dev/null 2>&1; then
  echo "pre-commit: launchd service $launchd_target is still loaded after bootout" >&2
  exit 1
fi

bootstrap_ok=0
for _attempt in $(seq 1 3); do
  if launchctl bootstrap "$launchd_domain" "$user_plist"; then
    bootstrap_ok=1
    break
  fi
  sleep 0.5
done

if [[ "$bootstrap_ok" -ne 1 ]]; then
  echo "pre-commit: failed to bootstrap $user_plist into $launchd_domain" >&2
  exit 1
fi

launchctl kickstart -k "$launchd_target"

service_state=$(launchctl print "$launchd_target")
if ! grep -F "program = $installed_binary" <<<"$service_state" >/dev/null; then
  echo "pre-commit: launchd is not pointing at $installed_binary" >&2
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "pre-commit: curl not available, skipping /health verification"
  exit 0
fi

echo "pre-commit: waiting for $health_url"
for _attempt in $(seq 1 20); do
  if health_json=$(curl -fsS "$health_url"); then
    echo "pre-commit: /health => $health_json"
    exit 0
  fi
  sleep 0.25
done

echo "pre-commit: launchd reload completed but /health did not come up at $health_url" >&2
exit 1
