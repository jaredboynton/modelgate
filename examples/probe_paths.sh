#!/usr/bin/env bash
set -u
HOSTS=(
  "codeium=https://server.codeium.com"
  "self-serve=https://server.self-serve.windsurf.com"
)
PATHS=(
  "/exa.api_server_pb.ApiServerService/AssignModel"
  "/exa.api_server_pb.ApiServerService/AssignModelV2"
  "/exa.cascade_pb.CascadeService/AssignModel"
  "/exa.api_server_pb.ApiServerService/Assign"
  "/exa.cascade_v1_pb.CascadeService/AssignModel"
)
for h in "${HOSTS[@]}"; do
  name="${h%%=*}"
  base="${h#*=}"
  for p in "${PATHS[@]}"; do
    echo "=== $name  $p ==="
    /usr/bin/curl -sS -o /dev/null -w "status=%{http_code} size=%{size_download}\n" \
      -X POST \
      -H 'content-type: application/proto' \
      -H 'accept: application/proto' \
      -H 'connect-protocol-version: 1' \
      --data-binary '@/dev/null' \
      "$base$p"
  done
  echo
done
