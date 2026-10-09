#!/bin/sh
set -eu

script_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
avenys_directory=$(CDPATH= cd -- "$script_directory/.." && pwd)
schema_path="$avenys_directory/protocol/owl_avenys.proto"
descriptor_path=${1:-"${TMPDIR:-/tmp}/owl_avenys_schema.pb"}
workspace_schema_path="$avenys_directory/../protocol/owl_avenys.proto"
workspace_descriptor_path="$descriptor_path.workspace"

if ! command -v protoc >/dev/null 2>&1; then
    echo "error: protoc is required to validate protocol/owl_avenys.proto" >&2
    exit 1
fi

protoc \
    --proto_path="$avenys_directory/protocol" \
    --descriptor_set_out="$descriptor_path" \
    "$schema_path"

if [ -f "$workspace_schema_path" ]; then
    trap 'rm -f "$workspace_descriptor_path"' EXIT HUP INT TERM
    protoc \
        --proto_path="$(dirname -- "$workspace_schema_path")" \
        --descriptor_set_out="$workspace_descriptor_path" \
        "$workspace_schema_path"
    if ! cmp -s "$descriptor_path" "$workspace_descriptor_path"; then
        echo "error: repository and Arch workspace protocol schemas differ" >&2
        exit 1
    fi
fi

printf 'validated %s with %s\n' \
    "$schema_path" "$(protoc --version)"
