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

is_allowed_fixture() {
  case "$1" in
    tests/*|*/tests/*|fixtures/*|*/fixtures/*|testdata/*|*/testdata/*|docs/*|README.md|AGENTS.md|LAYERS.md|PLANNING/*|.harness/*)
      return 0
      ;;
  esac
  return 1
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
    *.netrc|*/.netrc|*.npmrc|*/.npmrc|*.pypirc|*/.pypirc) reason="tool credential file" ;;
  esac

  if [ -n "$reason" ]; then
    if is_allowed_fixture "$path"; then
      return 0
    fi
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
  while IFS='\t' read -r path reason; do
    printf '  - %s (%s)\n' "$path" "$reason" >&2
  done < "$failures"
  echo "Remediation: remove the credential file from git, rotate exposed material if needed, and commit a redacted template instead." >&2
  exit 1
fi

echo "OK: no obvious tracked/staged secret marker filenames found."
