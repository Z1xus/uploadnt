#!/bin/sh
# runs the image for the newest commit on main, only if github attests that main's ci built it
set -e
cd "$(dirname "$0")"
repo=${REPO:?set REPO to the github repo whose ci builds the image}
image=ghcr.io/$(echo "$repo" | tr '[:upper:]' '[:lower:]')
head=$(gh api "repos/$repo/commits/main" --jq .sha)
docker pull -q "$image:$head" >/dev/null 2>&1 || exit 0
digest=$(docker image inspect --format '{{index .RepoDigests 0}}' "$image:$head")
grep -qx "IMAGE=$digest" .env 2>/dev/null && exit 0
gh attestation verify "oci://$digest" --repo "$repo" --signer-workflow "$repo/.github/workflows/ci.yml" \
  --source-ref refs/heads/main --source-digest "$head" >/dev/null
IMAGE=$digest docker compose up -d
echo "IMAGE=$digest" > .env
echo "running $digest"
