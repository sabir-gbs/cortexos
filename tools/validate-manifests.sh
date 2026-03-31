#!/usr/bin/env bash
# validate-manifests.sh — Validate manifest.json files against the SPEC 21 / SDK schema.
#
# Checks all required fields from the AppManifest interface in
# packages/sdk/src/types/manifest.ts and docs/specs/21_sdk_manifest_third_party_app_platform.md.
#
# Usage:
#   ./tools/validate-manifests.sh                       # validate all manifests
#   ./tools/validate-manifests.sh --manifest <file>     # validate a single manifest file
#   ./tools/validate-manifests.sh --id <id>             # validate manifest by app ID
#   ./tools/validate-manifests.sh --app-dir <dir>       # validate manifest in a specific app directory
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
errors=0
total=0

# Known permission types from SDK PermissionType enum
KNOWN_PERMISSIONS="files.read files.write clipboard notifications ai settings search network hardware.camera hardware.microphone hardware.location"

# Known categories from SDK AppCategory enum (system is first-party alias)
KNOWN_CATEGORIES="productivity communication development media education finance health games utilities other system"

# Length limits from SDK constants
MAX_ID_LENGTH=128
MAX_NAME_LENGTH=64
MAX_DESCRIPTION_LENGTH=512
MAX_AUTHOR_NAME_LENGTH=64
MAX_PERMISSION_REASON_LENGTH=256

# ---------------------------------------------------------------------------
# CLI argument parsing
# ---------------------------------------------------------------------------
MANIFEST_FILE=""
APP_ID=""
APP_DIR=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --manifest)
            MANIFEST_FILE="$2"
            shift 2
            ;;
        --id)
            APP_ID="$2"
            shift 2
            ;;
        --app-dir)
            APP_DIR="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [--manifest <file>] [--id <id>] [--app-dir <dir>]"
            echo ""
            echo "  No arguments    Validate all manifest.json files in apps/"
            echo "  --manifest FILE Validate a specific manifest.json file"
            echo "  --id ID         Validate manifest by app ID (e.g. com.cortexos.calculator)"
            echo "  --app-dir DIR   Validate manifest in a specific app directory (relative to repo root)"
            exit 0
            ;;
        *)
            echo "ERROR: Unknown argument: $1"
            exit 1
            ;;
    esac
done

# ---------------------------------------------------------------------------
# Manifest validation logic (jq-based)
# ---------------------------------------------------------------------------

# Validate a single manifest.json file.
# Args: $1 = absolute path to manifest.json, $2 = absolute path to app directory
validate_manifest() {
    local file="$1"
    local app_dir="$2"
    local app_name
    app_name="$(basename "$app_dir")"

    total=$((total + 1))

    if [ ! -f "$file" ]; then
        printf "%-40s FAIL\n" "$app_name"
        echo "  ERROR: manifest.json not found"
        errors=$((errors + 1))
        return 1
    fi

    # ---- Parse JSON ----
    local manifest_json
    if ! manifest_json=$(jq '.' "$file" 2>/dev/null); then
        printf "%-40s FAIL\n" "$app_name"
        echo "  ERROR: invalid JSON"
        errors=$((errors + 1))
        return 1
    fi

    local manifest_errors=0
    local error_msg=""

    # -------------------------------------------------------------------
    # Required top-level fields (from SDK AppManifest interface)
    # -------------------------------------------------------------------
    local required_fields="id name version entry_point description icon permissions capabilities min_os_version author"
    for field in $required_fields; do
        local has_field
        has_field=$(echo "$manifest_json" | jq --arg f "$field" 'has($f)')
        if [ "$has_field" != "true" ]; then
            error_msg="${error_msg}  ERROR: manifest is missing required field: ${field}\n"
            manifest_errors=$((manifest_errors + 1))
        fi
    done

    # -------------------------------------------------------------------
    # Validate 'id' — reverse-domain pattern, max length
    # -------------------------------------------------------------------
    local id_val
    id_val=$(echo "$manifest_json" | jq -r '.id // empty')
    if [ -n "$id_val" ]; then
        if ! echo "$id_val" | grep -qP '^[a-z0-9]+(\.[a-z0-9]+)*\.[a-z0-9-]+$'; then
            error_msg="${error_msg}  ERROR: manifest has invalid id: '${id_val}' (must match ^[a-z0-9]+(\\.[a-z0-9]+)*\\.[a-z0-9-]+\$)\n"
            manifest_errors=$((manifest_errors + 1))
        fi
        local id_len=${#id_val}
        if [ "$id_len" -gt "$MAX_ID_LENGTH" ]; then
            error_msg="${error_msg}  ERROR: manifest id exceeds maximum length (${id_len} > ${MAX_ID_LENGTH})\n"
            manifest_errors=$((manifest_errors + 1))
        fi
    fi

    # -------------------------------------------------------------------
    # Validate 'name' — non-empty, max length
    # -------------------------------------------------------------------
    local name_val
    name_val=$(echo "$manifest_json" | jq -r '.name // empty')
    if [ -n "$name_val" ]; then
        local name_len=${#name_val}
        if [ "$name_len" -eq 0 ]; then
            error_msg="${error_msg}  ERROR: manifest has empty name\n"
            manifest_errors=$((manifest_errors + 1))
        fi
        if [ "$name_len" -gt "$MAX_NAME_LENGTH" ]; then
            error_msg="${error_msg}  ERROR: manifest name exceeds maximum length (${name_len} > ${MAX_NAME_LENGTH})\n"
            manifest_errors=$((manifest_errors + 1))
        fi
    fi

    # -------------------------------------------------------------------
    # Validate 'version' — semver pattern
    # -------------------------------------------------------------------
    local version_val
    version_val=$(echo "$manifest_json" | jq -r '.version // empty')
    if [ -n "$version_val" ]; then
        if ! echo "$version_val" | grep -qP '^\d+\.\d+\.\d+$'; then
            error_msg="${error_msg}  ERROR: manifest has invalid version: '${version_val}' (must match semver ^\\d+\\.\\d+\\.\\d+\$)\n"
            manifest_errors=$((manifest_errors + 1))
        fi
    fi

    # -------------------------------------------------------------------
    # Validate 'min_os_version' — semver pattern
    # -------------------------------------------------------------------
    local min_ver
    min_ver=$(echo "$manifest_json" | jq -r '.min_os_version // empty')
    if [ -n "$min_ver" ]; then
        if ! echo "$min_ver" | grep -qP '^\d+\.\d+\.\d+$'; then
            error_msg="${error_msg}  ERROR: manifest has invalid min_os_version: '${min_ver}' (must match semver ^\\d+\\.\\d+\\.\\d+\$)\n"
            manifest_errors=$((manifest_errors + 1))
        fi
    fi

    # -------------------------------------------------------------------
    # Validate 'entry_point' — must reference an existing file
    # -------------------------------------------------------------------
    local entry_point
    entry_point=$(echo "$manifest_json" | jq -r '.entry_point // empty')
    if [ -n "$entry_point" ] && [ -d "$app_dir" ]; then
        if [ ! -f "${app_dir}/${entry_point}" ]; then
            error_msg="${error_msg}  ERROR: manifest entry_point '${entry_point}' does not exist at ${app_dir}/${entry_point}\n"
            manifest_errors=$((manifest_errors + 1))
        fi
    fi

    # -------------------------------------------------------------------
    # Validate 'icon' — must be PNG, SVG, or ICO
    # -------------------------------------------------------------------
    local icon_val
    icon_val=$(echo "$manifest_json" | jq -r '.icon // empty')
    if [ -n "$icon_val" ]; then
        if [[ "$icon_val" != *.png && "$icon_val" != *.svg && "$icon_val" != *.ico ]]; then
            error_msg="${error_msg}  ERROR: manifest icon '${icon_val}' must be PNG, SVG, or ICO\n"
            manifest_errors=$((manifest_errors + 1))
        fi
    fi

    # -------------------------------------------------------------------
    # Validate 'description' — max length
    # -------------------------------------------------------------------
    local desc_len
    desc_len=$(echo "$manifest_json" | jq -r '.description // ""' | wc -c)
    desc_len=$((desc_len - 1))  # wc -c includes trailing newline
    if [ "$desc_len" -gt "$MAX_DESCRIPTION_LENGTH" ]; then
        error_msg="${error_msg}  ERROR: manifest description exceeds maximum length (${desc_len} > ${MAX_DESCRIPTION_LENGTH})\n"
        manifest_errors=$((manifest_errors + 1))
    fi

    # -------------------------------------------------------------------
    # Validate 'permissions' — array of objects with known permission types
    # -------------------------------------------------------------------
    local perm_type
    perm_type=$(echo "$manifest_json" | jq -r '.permissions | type')
    if [ "$perm_type" != "array" ]; then
        error_msg="${error_msg}  ERROR: 'permissions' must be an array, got: ${perm_type}\n"
        manifest_errors=$((manifest_errors + 1))
    else
        local perm_count
        perm_count=$(echo "$manifest_json" | jq '.permissions | length')
        local i=0
        while [ "$i" -lt "$perm_count" ]; do
            local perm_obj_type
            perm_obj_type=$(echo "$manifest_json" | jq -r ".permissions[$i] | type")
            if [ "$perm_obj_type" != "object" ]; then
                error_msg="${error_msg}  ERROR: permissions[$i] must be an object, got: ${perm_obj_type}\n"
                manifest_errors=$((manifest_errors + 1))
            else
                # Check permission name is known
                local perm_name
                perm_name=$(echo "$manifest_json" | jq -r ".permissions[$i].permission // empty")
                if [ -z "$perm_name" ]; then
                    error_msg="${error_msg}  ERROR: permissions[$i] is missing required field: permission\n"
                    manifest_errors=$((manifest_errors + 1))
                else
                    local perm_known=false
                    for known_perm in $KNOWN_PERMISSIONS; do
                        if [ "$perm_name" = "$known_perm" ]; then
                            perm_known=true
                            break
                        fi
                    done
                    if [ "$perm_known" = "false" ]; then
                        error_msg="${error_msg}  ERROR: permissions[$i].permission is unknown: '${perm_name}'\n"
                        manifest_errors=$((manifest_errors + 1))
                    fi
                fi

                # Check 'required' field exists and is boolean
                local has_required
                has_required=$(echo "$manifest_json" | jq ".permissions[$i] | has(\"required\")")
                if [ "$has_required" != "true" ]; then
                    error_msg="${error_msg}  ERROR: permissions[$i] is missing required field: required\n"
                    manifest_errors=$((manifest_errors + 1))
                else
                    local req_type
                    req_type=$(echo "$manifest_json" | jq -r ".permissions[$i].required | type")
                    if [ "$req_type" != "boolean" ]; then
                        error_msg="${error_msg}  ERROR: permissions[$i].required must be a boolean, got: ${req_type}\n"
                        manifest_errors=$((manifest_errors + 1))
                    fi
                fi

                # Check 'reason' field exists
                local has_reason
                has_reason=$(echo "$manifest_json" | jq ".permissions[$i] | has(\"reason\")")
                if [ "$has_reason" != "true" ]; then
                    error_msg="${error_msg}  ERROR: permissions[$i] is missing required field: reason\n"
                    manifest_errors=$((manifest_errors + 1))
                else
                    local reason_len
                    reason_len=$(echo "$manifest_json" | jq -r ".permissions[$i].reason" | wc -c)
                    reason_len=$((reason_len - 1))
                    if [ "$reason_len" -gt "$MAX_PERMISSION_REASON_LENGTH" ]; then
                        error_msg="${error_msg}  ERROR: permissions[$i].reason exceeds maximum length (${reason_len} > ${MAX_PERMISSION_REASON_LENGTH})\n"
                        manifest_errors=$((manifest_errors + 1))
                    fi
                fi
            fi
            i=$((i + 1))
        done
    fi

    # -------------------------------------------------------------------
    # Validate 'capabilities' — object with required sub-fields
    # -------------------------------------------------------------------
    local cap_type
    cap_type=$(echo "$manifest_json" | jq -r '.capabilities | type')
    if [ "$cap_type" == "object" ]; then
        local cap_fields="ai background_execution autostart max_file_size system_tray file_handlers"
        for cap_field in $cap_fields; do
            local has_cap
            has_cap=$(echo "$manifest_json" | jq --arg cf "$cap_field" '.capabilities | has($cf)')
            if [ "$has_cap" != "true" ]; then
                error_msg="${error_msg}  ERROR: manifest is missing required field: capabilities.${cap_field}\n"
                manifest_errors=$((manifest_errors + 1))
            fi
        done

        # Boolean checks
        for bool_field in ai background_execution autostart system_tray; do
            local bf_type
            bf_type=$(echo "$manifest_json" | jq -r ".capabilities.${bool_field} | type")
            if [ "$bf_type" != "boolean" ] && [ "$bf_type" != "null" ]; then
                error_msg="${error_msg}  ERROR: capabilities.${bool_field} must be a boolean, got: ${bf_type}\n"
                manifest_errors=$((manifest_errors + 1))
            fi
        done

        # max_file_size must be a number
        local mfs_type
        mfs_type=$(echo "$manifest_json" | jq -r '.capabilities.max_file_size | type')
        if [ "$mfs_type" != "number" ] && [ "$mfs_type" != "null" ]; then
            error_msg="${error_msg}  ERROR: capabilities.max_file_size must be a number, got: ${mfs_type}\n"
            manifest_errors=$((manifest_errors + 1))
        fi

        # file_handlers must be an array
        local fh_type
        fh_type=$(echo "$manifest_json" | jq -r '.capabilities.file_handlers | type')
        if [ "$fh_type" != "array" ] && [ "$fh_type" != "null" ]; then
            error_msg="${error_msg}  ERROR: capabilities.file_handlers must be an array, got: ${fh_type}\n"
            manifest_errors=$((manifest_errors + 1))
        fi

        # Validate individual file handlers
        if [ "$fh_type" == "array" ]; then
            local fh_count
            fh_count=$(echo "$manifest_json" | jq '.capabilities.file_handlers | length')
            local j=0
            while [ "$j" -lt "$fh_count" ]; do
                for fh_req in extension mime_type label; do
                    local has_fh
                    has_fh=$(echo "$manifest_json" | jq ".capabilities.file_handlers[$j] | has(\"${fh_req}\")")
                    if [ "$has_fh" != "true" ]; then
                        error_msg="${error_msg}  ERROR: capabilities.file_handlers[$j] is missing required field: ${fh_req}\n"
                        manifest_errors=$((manifest_errors + 1))
                    fi
                done
                j=$((j + 1))
            done
        fi
    elif [ "$cap_type" == "null" ]; then
        # Already flagged as missing required field above
        :
    else
        error_msg="${error_msg}  ERROR: 'capabilities' must be an object, got: ${cap_type}\n"
        manifest_errors=$((manifest_errors + 1))
    fi

    # -------------------------------------------------------------------
    # Validate 'author' — object with required 'name' sub-field
    # -------------------------------------------------------------------
    local author_type
    author_type=$(echo "$manifest_json" | jq -r '.author | type')
    if [ "$author_type" == "object" ]; then
        local has_author_name
        has_author_name=$(echo "$manifest_json" | jq '.author | has("name")')
        if [ "$has_author_name" != "true" ]; then
            error_msg="${error_msg}  ERROR: manifest is missing required field: author.name\n"
            manifest_errors=$((manifest_errors + 1))
        else
            local author_name_val
            author_name_val=$(echo "$manifest_json" | jq -r '.author.name // empty')
            if [ -z "$author_name_val" ]; then
                error_msg="${error_msg}  ERROR: author.name must not be empty\n"
                manifest_errors=$((manifest_errors + 1))
            else
                local an_len=${#author_name_val}
                if [ "$an_len" -gt "$MAX_AUTHOR_NAME_LENGTH" ]; then
                    error_msg="${error_msg}  ERROR: author.name exceeds maximum length (${an_len} > ${MAX_AUTHOR_NAME_LENGTH})\n"
                    manifest_errors=$((manifest_errors + 1))
                fi
            fi
        fi
    elif [ "$author_type" == "null" ]; then
        # Already flagged as missing required field above
        :
    else
        error_msg="${error_msg}  ERROR: 'author' must be an object, got: ${author_type}\n"
        manifest_errors=$((manifest_errors + 1))
    fi

    # -------------------------------------------------------------------
    # Validate 'category' — optional but must be known if present
    # -------------------------------------------------------------------
    local cat_val
    cat_val=$(echo "$manifest_json" | jq -r '.category // empty')
    if [ -n "$cat_val" ]; then
        local cat_valid=false
        for known_cat in $KNOWN_CATEGORIES; do
            if [ "$cat_val" = "$known_cat" ]; then
                cat_valid=true
                break
            fi
        done
        if [ "$cat_valid" = "false" ]; then
            error_msg="${error_msg}  ERROR: manifest has unknown category: '${cat_val}' (known: ${KNOWN_CATEGORIES})\n"
            manifest_errors=$((manifest_errors + 1))
        fi
    fi

    # -------------------------------------------------------------------
    # Game-specific checks (optional 'game' block)
    # -------------------------------------------------------------------
    local has_game
    has_game=$(echo "$manifest_json" | jq 'has("game")')
    if [ "$has_game" = "true" ]; then
        local diff_type
        diff_type=$(echo "$manifest_json" | jq -r '.game.difficulties | type')
        if [ "$diff_type" != "array" ]; then
            error_msg="${error_msg}  ERROR: 'game.difficulties' must be an array, got: ${diff_type}\n"
            manifest_errors=$((manifest_errors + 1))
        else
            local diff_count
            diff_count=$(echo "$manifest_json" | jq '.game.difficulties | length')
            if [ "$diff_count" -eq 0 ]; then
                error_msg="${error_msg}  ERROR: 'game.difficulties' must be a non-empty array\n"
                manifest_errors=$((manifest_errors + 1))
            fi
        fi

        local default_diff
        default_diff=$(echo "$manifest_json" | jq -r '.game.defaultDifficulty // empty')
        if [ -z "$default_diff" ]; then
            error_msg="${error_msg}  ERROR: missing or empty 'game.defaultDifficulty'\n"
            manifest_errors=$((manifest_errors + 1))
        else
            local found_diff
            found_diff=$(echo "$manifest_json" | jq --arg d "$default_diff" '[.game.difficulties[] | select(. == $d)] | length')
            if [ "$found_diff" -eq 0 ]; then
                error_msg="${error_msg}  ERROR: 'game.defaultDifficulty' value \"${default_diff}\" not found in game.difficulties\n"
                manifest_errors=$((manifest_errors + 1))
            fi
        fi
    fi

    # -------------------------------------------------------------------
    # Output result
    # -------------------------------------------------------------------
    if [ "$manifest_errors" -gt 0 ]; then
        printf "%-40s FAIL\n" "$app_name"
        errors=$((errors + manifest_errors))
        printf "%b" "$error_msg"
    else
        printf "%-40s PASS\n" "$app_name"
    fi
}

echo "=== CortexOS Manifest Validator (SPEC 21 Schema) ==="
echo ""

# ---------------------------------------------------------------------------
# Determine which manifests to validate based on CLI arguments
# ---------------------------------------------------------------------------

if [ -n "$MANIFEST_FILE" ]; then
    # --manifest: validate a specific manifest file
    if [[ "$MANIFEST_FILE" != /* ]]; then
        MANIFEST_FILE="$REPO_ROOT/$MANIFEST_FILE"
    fi
    if [ ! -f "$MANIFEST_FILE" ]; then
        echo "ERROR: manifest file not found: $MANIFEST_FILE"
        exit 1
    fi
    app_dir=$(dirname "$MANIFEST_FILE")
    validate_manifest "$MANIFEST_FILE" "$app_dir"

elif [ -n "$APP_ID" ]; then
    # --id: find manifest by app ID
    found=false
    for candidate in "$REPO_ROOT"/apps/*/manifest.json "$REPO_ROOT"/apps/games/*/manifest.json; do
        [ -f "$candidate" ] || continue
        candidate_id=$(jq -r '.id // empty' "$candidate" 2>/dev/null)
        if [ "$candidate_id" = "$APP_ID" ]; then
            found=true
            app_dir=$(dirname "$candidate")
            validate_manifest "$candidate" "$app_dir"
            break
        fi
    done
    if [ "$found" = "false" ]; then
        echo "ERROR: no manifest found with id: $APP_ID"
        errors=$((errors + 1))
        total=$((total + 1))
    fi

elif [ -n "$APP_DIR" ]; then
    # --app-dir: validate manifest in a specific directory
    if [[ "$APP_DIR" != /* ]]; then
        APP_DIR="$REPO_ROOT/$APP_DIR"
    fi
    manifest="$APP_DIR/manifest.json"
    validate_manifest "$manifest" "$APP_DIR"

else
    # No arguments: validate all manifests
    for app_dir in "$REPO_ROOT"/apps/*/ "$REPO_ROOT"/apps/games/*/; do
        [ -d "$app_dir" ] || continue
        name="$(basename "$app_dir")"
        if [ "$name" = "shared" ] || [ "$name" = "games" ]; then
            continue
        fi
        manifest="$app_dir/manifest.json"
        if [ ! -f "$manifest" ]; then
            printf "%-40s WARN\n" "$name"
            echo "  no manifest.json found, skipping"
            continue
        fi
        validate_manifest "$manifest" "$app_dir"
    done
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
if [ "$errors" -gt 0 ]; then
    echo "FAILED: ${errors} error(s) found across ${total} manifest(s)"
    exit 1
else
    echo "OK: validated ${total} manifest(s), 0 errors"
    exit 0
fi
