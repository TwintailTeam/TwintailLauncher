#!/usr/bin/env bash
set -euo pipefail

# Required env vars:
#   TOLGEE_URL        - Tolgee instance URL
#   TOLGEE_API_KEY    - Project API key with translations.view scope
#   TOLGEE_PROJECT_ID - Numeric project ID
#
# Optional env vars:
#   LOCALES_DIR       - Repository-relative target directory for locale files
#   EXCLUDE_LANGUAGES - Space-separated local stems or tags to skip
#   DRY_RUN           - Set to "true" to print actions without executing

: "${TOLGEE_URL:?TOLGEE_URL is required}"
: "${TOLGEE_API_KEY:?TOLGEE_API_KEY is required}"
: "${TOLGEE_PROJECT_ID:?TOLGEE_PROJECT_ID is required}"

TOLGEE_URL="${TOLGEE_URL%/}"
LOCALES_DIR="${LOCALES_DIR:-src-tauri/resources/locales}"
LOCALES_DIR="${LOCALES_DIR%/}"
EXCLUDE_LANGUAGES="${EXCLUDE_LANGUAGES:-en_NEKO}"
DRY_RUN="${DRY_RUN:-false}"

REPO_ROOT=$(git rev-parse --show-toplevel)
LOCALES_ABS="$REPO_ROOT/$LOCALES_DIR"

if [[ ! -d "$LOCALES_ABS" ]]; then
  echo "Locales directory not found: $LOCALES_ABS" >&2
  exit 1
fi

SYNC_TMP_DIR=$(mktemp -d)
trap 'rm -rf "$SYNC_TMP_DIR"' EXIT

log()  { printf '[sync] %s\n' "$*"; }
warn() { printf '[sync] WARNING: %s\n' "$*" >&2; }

# Tolgee uses tags such as de-DE, while the repository uses de_DE.json.
to_local_filename() {
  local remote_tag="$1"
  printf '%s' "${remote_tag//-/_}"
}

locale_relative_path() {
  local local_filename="$1"
  printf '%s/%s.json' "$LOCALES_DIR" "$local_filename"
}

locale_absolute_path() {
  local local_filename="$1"
  printf '%s/%s' "$REPO_ROOT" "$(locale_relative_path "$local_filename")"
}

is_excluded() {
  local local_filename="$1"
  local excluded

  for excluded in $EXCLUDE_LANGUAGES; do
    excluded="${excluded//-/_}"
    [[ "$local_filename" == "$excluded" ]] && return 0
  done

  return 1
}

export_language() {
  local remote_tag="$1"
  local local_filename="$2"

  local url="$TOLGEE_URL/v2/projects/$TOLGEE_PROJECT_ID/export"
  url+="?format=JSON&structureDelimiter=.&supportArrays=true&zip=false"
  url+="&languages=$remote_tag"

  local response_file="$SYNC_TMP_DIR/${remote_tag}.export"
  local headers_file="$SYNC_TMP_DIR/${remote_tag}.headers"
  local http
  local curl_rc=0
  http=$(curl -s -w "%{http_code}" -o "$response_file" -D "$headers_file" \
    -H "X-API-Key: $TOLGEE_API_KEY" \
    "$url") || curl_rc=$?

  if [[ "$http" != "200" ]]; then
    warn "  Export failed for $remote_tag (HTTP $http, curl exit $curl_rc)"
    warn "  $(head -c 500 "$response_file" 2>/dev/null)"
    if ! grep -qi '^x-tolgee-version:' "$headers_file"; then
      warn "  No x-tolgee-version header: this response is from a proxy or WAF, not Tolgee"
    fi
    return 1
  fi

  # A single-language Tolgee export can be either a ZIP or direct JSON.
  local extract_dir="$SYNC_TMP_DIR/extract_${local_filename}"
  mkdir -p "$extract_dir"

  if file "$response_file" | grep -qi zip; then
    unzip -qo "$response_file" -d "$extract_dir" 2>/dev/null || true
  else
    cp "$response_file" "$extract_dir/${local_filename}.json"
  fi

  # Match the requested language by name. Taking the first file found would write
  # another language's strings into this locale if the export ever returns more
  # than one file.
  local json
  json=$(find "$extract_dir" -maxdepth 2 -type f \
    \( -name "${local_filename}.json" -o -name "${remote_tag}.json" \) -print -quit)

  if [[ -z "$json" ]]; then
    warn "  No JSON for $remote_tag in export (contains: $(
      find "$extract_dir" -maxdepth 2 -type f -printf '%f ' 2>/dev/null
    ))"
    return 1
  fi

  # Preserve metadata from the existing file, or generate it for a new locale.
  if ! jq -e '.metadata' "$json" >/dev/null 2>&1; then
    local target
    target=$(locale_absolute_path "$local_filename")

    if [[ -f "$target" ]]; then
      local metadata
      metadata=$(jq '{metadata: .metadata}' "$target")
      jq -s '.[0] * .[1]' \
        "$json" \
        <(printf '%s\n' "$metadata") \
        >"$json.tmp"
      mv "$json.tmp" "$json"
    else
      local language_name
      language_name=$(printf '%s\n' "$STATS" | jq -r \
        --arg remote_tag "$remote_tag" '
          .languageStats[]
          | select(.languageTag == $remote_tag)
          | .languageName // .languageTag
        ')

      jq \
        --arg code "$local_filename" \
        --arg name "$language_name" \
        '{metadata: {code: $code, display_name: $name}} + .' \
        "$json" \
        >"$json.tmp"
      mv "$json.tmp" "$json"
    fi
  fi

  cp "$json" "$SYNC_TMP_DIR/${local_filename}.json"
}

log "Fetching project stats..."

STATS_BODY="$SYNC_TMP_DIR/stats.json"
STATS_HEADERS="$SYNC_TMP_DIR/stats.headers"
CURL_RC=0

STATS_HTTP=$(curl -s -o "$STATS_BODY" -D "$STATS_HEADERS" -w '%{http_code}' \
  -H "X-API-Key: $TOLGEE_API_KEY" \
  "$TOLGEE_URL/v2/projects/$TOLGEE_PROJECT_ID/stats"
) || CURL_RC=$?

if [[ "$STATS_HTTP" != "200" ]]; then
  echo "Failed to fetch project stats (HTTP $STATS_HTTP, curl exit $CURL_RC)" >&2
  echo "--- response headers ---" >&2
  cat "$STATS_HEADERS" >&2 2>/dev/null || true
  echo "--- response body ---" >&2
  head -c 2000 "$STATS_BODY" >&2 2>/dev/null || true
  echo >&2

  # Tolgee stamps every response with x-tolgee-version. Without it the reply came
  # from something in front of Tolgee, so the status says nothing about the key.
  if ! grep -qi '^x-tolgee-version:' "$STATS_HEADERS"; then
    warn "No x-tolgee-version header: this response is from a proxy or WAF, not Tolgee"
  fi

  exit 1
fi

STATS=$(cat "$STATS_BODY")

# Evaluate each language independently; ignore the overall project percentage.
READY=$(printf '%s\n' "$STATS" | jq -r '
  .languageStats[]
  | select(.reviewedPercentage == 100 and .reviewedKeyCount > 0)
  | .languageTag
')

if [[ -z "$READY" ]]; then
  log "No languages at 100% reviewed. Nothing to do."
  exit 0
fi

# Aggregate PR branches are sync/locales-<timestamp>. Detect an existing PR by
# its exact changed locale path. Fail closed to avoid duplicate daily PRs.
if ! OPEN_SYNC_FILES=$(
  gh pr list \
    --state open \
    --base master \
    --limit 1000 \
    --json headRefName,files |
    jq -r '
      .[]
      | select(.headRefName | startswith("sync/locales-"))
      | .files[]?.path
    '
); then
  warn "Failed to inspect open sync PRs; aborting to avoid duplicate PRs"
  exit 1
fi

has_open_sync_pr_for_language() {
  local local_filename="$1"
  local expected_path
  expected_path=$(locale_relative_path "$local_filename")

  grep -Fqx "$expected_path" <<<"$OPEN_SYNC_FILES"
}

CHANGED_LANGS=()
CHANGED_FILES=()
CHANGED_KEYS=0

for REMOTE_TAG in $READY; do
  LOCAL_FILENAME=$(to_local_filename "$REMOTE_TAG")

  is_excluded "$LOCAL_FILENAME" && continue

  RELATIVE_TARGET=$(locale_relative_path "$LOCAL_FILENAME")
  LOCAL_TARGET=$(locale_absolute_path "$LOCAL_FILENAME")
  TEMP_JSON="$SYNC_TMP_DIR/${LOCAL_FILENAME}.json"

  log "Processing $REMOTE_TAG as $RELATIVE_TARGET..."

  if has_open_sync_pr_for_language "$LOCAL_FILENAME"; then
    log "  Open PR already contains $REMOTE_TAG, skipping"
    continue
  fi

  if ! export_language "$REMOTE_TAG" "$LOCAL_FILENAME"; then
    continue
  fi

  if [[ -f "$LOCAL_TARGET" ]] &&
    diff -q "$TEMP_JSON" "$LOCAL_TARGET" >/dev/null 2>&1; then
    log "  $REMOTE_TAG is up to date"
    continue
  fi

  KEY_COUNT=$(jq '
    with_entries(select(.key != "metadata"))
    | [paths(scalars)]
    | length
  ' "$TEMP_JSON")

  CHANGED_LANGS+=("$REMOTE_TAG")
  CHANGED_FILES+=("$LOCAL_FILENAME")
  CHANGED_KEYS=$((CHANGED_KEYS + KEY_COUNT))
  log "  $REMOTE_TAG has changes ($KEY_COUNT keys)"
done

if [[ ${#CHANGED_LANGS[@]} -eq 0 ]]; then
  log "All ready languages are up to date or already have an open PR."
  exit 0
fi

if [[ "$DRY_RUN" == "true" ]]; then
  log "DRY RUN - would create PR for: ${CHANGED_LANGS[*]}"
  exit 0
fi

LANG_LIST=$(printf '%s, ' "${CHANGED_LANGS[@]}")
LANG_LIST="${LANG_LIST%, }"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
BRANCH="sync/locales-$TIMESTAMP"

git -C "$REPO_ROOT" checkout -b "$BRANCH"

for LOCAL_FILENAME in "${CHANGED_FILES[@]}"; do
  cp "$SYNC_TMP_DIR/${LOCAL_FILENAME}.json" "$(locale_absolute_path "$LOCAL_FILENAME")"
  git -C "$REPO_ROOT" add "$(locale_relative_path "$LOCAL_FILENAME")"
done

git -C "$REPO_ROOT" commit -m "feat(i18n): sync translations from https://localize.tukandev.com

Synced languages: ${CHANGED_LANGS[*]}
Total keys: $CHANGED_KEYS
Automated sync via sync_locales.sh"

git -C "$REPO_ROOT" push -fu origin "$BRANCH"

BODY="Auto-generated translation sync from https://localize.tukandev.com

**Languages synced:** ${#CHANGED_LANGS[@]}
**Total keys:** $CHANGED_KEYS

| Language | Keys |
|----------|------|
"

for INDEX in "${!CHANGED_LANGS[@]}"; do
  REMOTE_TAG="${CHANGED_LANGS[$INDEX]}"
  LOCAL_FILENAME="${CHANGED_FILES[$INDEX]}"

  KEY_COUNT=$(jq '
    with_entries(select(.key != "metadata"))
    | [paths(scalars)]
    | length
  ' "$SYNC_TMP_DIR/${LOCAL_FILENAME}.json")

  BODY+="| \`$REMOTE_TAG\` | $KEY_COUNT |
"
done

BODY+="
---

> Created by [sync_locales.sh](scripts/sync_locales.sh)"

gh pr create \
  --title "feat(i18n): sync translations - $LANG_LIST" \
  --body "$BODY" \
  --head "$BRANCH" \
  --base master

log "Done! PR created with ${#CHANGED_LANGS[@]} language(s): $LANG_LIST"
