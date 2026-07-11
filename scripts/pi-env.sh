#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOCAL_ALTOIDS_ENV="${SCRIPT_DIR}/../../altoids/termpaper.env"

if [[ -f "${LOCAL_ALTOIDS_ENV}" ]]; then
  # shellcheck source=/dev/null
  source "${LOCAL_ALTOIDS_ENV}"
fi

require_pi_config() {
  local missing=0

  if [[ -z "${PI_HOST:-}" ]]; then
    echo "PI_HOST is not set. Export it or create ../altoids/termpaper.env." >&2
    missing=1
  fi

  if [[ -z "${PI_DIR:-}" ]]; then
    echo "PI_DIR is not set. Export it or create ../altoids/termpaper.env." >&2
    missing=1
  fi

  if [[ "${missing}" != "0" ]]; then
    exit 2
  fi
}

require_pi_host() {
  if [[ -z "${PI_HOST:-}" ]]; then
    echo "PI_HOST is not set. Export it or create ../altoids/termpaper.env." >&2
    exit 2
  fi
}
