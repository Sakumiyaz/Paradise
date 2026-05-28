#!/usr/bin/env bash
set -Eeuo pipefail

if ! command -v go >/dev/null 2>&1; then
    printf 'go is required to install gitleaks and trufflehog\n' >&2
    exit 127
fi

go install github.com/zricethezav/gitleaks/v8@latest
go install github.com/trufflesecurity/trufflehog/v3@latest

printf 'installed release scanners into %s/bin\n' "$(go env GOPATH)"
