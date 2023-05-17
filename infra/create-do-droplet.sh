#!/bin/bash
set -euo pipefail

USERDATA_TEMPLATE="$(<./infra/userdata.sh.template)"
USERDATA="${USERDATA_TEMPLATE//\{\{DO_API_KEY\}\}/$DO_API_KEY}"

doctl compute droplet create \
    --image debian-10-x64 \
    --size s-1vcpu-1gb \
    --region sfo3 \
    --ssh-keys ${DO_SSH_FINGERPRINT} \
    --user-data "${USERDATA}" \
    github-code-fetcher-droplet
