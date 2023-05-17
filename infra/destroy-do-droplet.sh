#!/bin/bash
set -euo pipefail

doctl compute droplet delete --force github-code-fetcher-droplet