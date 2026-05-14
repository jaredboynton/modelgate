#!/usr/bin/env sh
set -eu

repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$repo_root"

failures=$(mktemp)
trap 'rm -f "$failures"' EXIT

list_paths() {
  git ls-files
  git diff --cached --name-only --diff-filter=ACMRT
}

check_path() {
  path=$1
  lower=$(printf '%s' "$path" | tr '[:upper:]' '[:lower:]')
  reason=

  case "$lower" in
    *id_rsa*|*id_dsa*|*id_ecdsa*|*id_ed25519*) reason="private SSH key filename" ;;
    *.pem|*.key|*.p12|*.pfx|*.jks|*.keystore) reason="private key/certificate store filename" ;;
    *service-account*.json|*service_account*.json|*gcloud*credentials*.json) reason="cloud service credential filename" ;;
    *credentials*.json|*credential*.json|*secrets*.json|*secret*.json|*tokens*.json|*token*.json|*auth.json|*oauth*.json|*cookies*.json|*cookie*.json|*session*.json) reason="credential capture filename" ;;
    *.netrc|*.npmrc|*.pypirc) reason="tool credential file" ;;
  esac

  if [ -n "$reason" ]; then
    printf '%s\t%s\n' "$path" "$reason" >> "$failures"
  fi
}

list_paths | sort -u | while IFS= read -r path; do
  [ -n "$path" ] || continue
  check_path "$path"
done

if [ -s "$failures" ]; then
  echo "GC hard fail: obvious secret or credential filenames are tracked/staged." >&2
  echo "This check uses filenames only and does not read secret contents." >&2
  while IFS="$(printf '\t')" read -r path reason; do
    printf '  - %s (%s)\n' "$path" "$reason" >&2
  done < "$failures"
  echo "Remediation: remove the credential file from git, rotate exposed material if needed, and commit a redacted template instead." >&2
  exit 1
fi

echo "OK: no obvious tracked/staged secret marker filenames found."
