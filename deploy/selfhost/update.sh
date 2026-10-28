#!/bin/sh
# runs the newest image only if github attests that the repo's ci built it
set -e
cd "$(dirname "$0")"
repo=${REPO:?set REPO to the github repo whose ci builds the image}
image=ghcr.io/$(echo "$repo" | tr '[:upper:]' '[:lower:]')
docker pull -q "$image:latest" >/dev/null
digest=$(docker image inspect --format '{{index .RepoDigests 0}}' "$image:latest")
grep -qx "IMAGE=$digest" .env 2>/dev/null && exit 0
gh attestation verify "oci://$digest" --repo "$repo" >/dev/null
echo "IMAGE=$digest" > .env
docker compose up -d
echo "running $digest"
