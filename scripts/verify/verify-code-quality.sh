#!/usr/bin/env bash
# RusTok — Верификация code quality
# Фаза 19: PII logging, hardcoded secrets, code metrics, dependency antipatterns
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$ROOT_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'
BOLD='\033[1m'

ERRORS=0
WARNINGS=0

header() { echo -e "\n${BOLD}=== $1 ===${NC}"; }
pass()   { echo -e "  ${GREEN}✓${NC} $1"; }
fail()   { echo -e "  ${RED}✗${NC} $1"; ERRORS=$((ERRORS + 1)); }
warn()   { echo -e "  ${YELLOW}!${NC} $1"; WARNINGS=$((WARNINGS + 1)); }

PROD_RS_PATHS=(
    "crates/rustok-core/src"
    "crates/rustok-content/src"
    "crates/rustok-commerce/src"
    "crates/rustok-blog/src"
    "crates/rustok-forum/src"
    "crates/rustok-pages/src"
    "crates/rustok-events/src"
    "crates/rustok-rbac/src"
    "crates/rustok-tenant/src"
    "crates/rustok-telemetry/src"
    "crates/rustok-iggy/src"
    "crates/rustok-outbox/src"
    "crates/rustok-index/src"
    "crates/alloy-scripting/src"
    "apps/server/src"
)

EXISTING=()
for p in "${PROD_RS_PATHS[@]}"; do
    [[ -d "$p" ]] && EXISTING+=("$p")
done

if [[ ${#EXISTING[@]} -eq 0 ]]; then
    echo -e "${YELLOW}No production code paths found. Skipping.${NC}"
    exit 0
fi

# ═══════════════════════════════════════════
# SECURITY
# ═══════════════════════════════════════════

# ─── 1. PII в логах ───
header "1. PII в логах (password, email, token в tracing)"

pii_in_logs=$(grep -rn 'tracing::\|info!\|debug!\|warn!\|error!\|trace!' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | grep -iE 'password|email|token|secret|credential' | grep -v "test\|// \|password_hash\|token_type\|email_verified\|password_reset\|token_expir" || true)
if [[ -n "$pii_in_logs" ]]; then
    count=$(echo "$pii_in_logs" | wc -l)
    fail "$count PII logging instance(s) found:"
    echo "$pii_in_logs" | head -15
else
    pass "No PII found in tracing/logging calls"
fi

# ─── 2. Hardcoded secrets ───
header "2. Hardcoded secrets"

# Look for string assignments with suspicious names
hardcoded=$(grep -rn 'const\|static\|let' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | grep -iE 'secret|password|api_key|private_key' | grep '=' | grep '"' | grep -v "test\|example\|env::var\|config\.\|// \|///\|fn \|pub fn\|expect\|env!" || true)
if [[ -n "$hardcoded" ]]; then
    count=$(echo "$hardcoded" | wc -l)
    fail "$count potential hardcoded secret(s):"
    echo "$hardcoded" | head -10
else
    pass "No hardcoded secrets detected"
fi

# Check for .env files in git
env_in_git=$(git ls-files '*.env' '.env*' 2>/dev/null | grep -v "example\|sample\|template\|dev\.example" || true)
if [[ -n "$env_in_git" ]]; then
    fail ".env file(s) tracked by git:"
    echo "$env_in_git"
else
    pass "No .env files in git (only examples)"
fi

# ─── 3. Entities exposed directly in API responses ───
header "3. Entities exposed directly in API responses"

# Check if Model structs are returned in Json<> responses
direct_model_return=$(grep -rn 'Json<.*Model>\|Json<.*Entity>\|Json<Vec<.*Model>>' "apps/server/src/controllers" --include="*.rs" 2>/dev/null | grep -v "test\|// " || true)
if [[ -n "$direct_model_return" ]]; then
    count=$(echo "$direct_model_return" | wc -l)
    warn "$count direct Model/Entity return(s) in controllers (should use Response DTOs):"
    echo "$direct_model_return" | head -10
else
    pass "Controllers don't return raw Model/Entity types"
fi

# ═══════════════════════════════════════════
# CODE METRICS
# ═══════════════════════════════════════════

# ─── 4. Модули > 500 строк ───
header "4. Файлы > 500 строк"

large_files=""
for dir in "${EXISTING[@]}"; do
    while IFS= read -r file; do
        lines=$(wc -l < "$file" 2>/dev/null || echo "0")
        if [[ $lines -gt 500 ]]; then
            large_files+="  $file: $lines lines"$'\n'
        fi
    done < <(find "$dir" -name "*.rs" -type f 2>/dev/null)
done

if [[ -n "$large_files" ]]; then
    count=$(echo "$large_files" | grep -c . || echo "0")
    warn "$count file(s) exceed 500 lines (consider splitting):"
    echo "$large_files" | sort -t: -k2 -rn | head -15
else
    pass "All files under 500 lines"
fi

# ─── 5. Функции > 60 строк ───
header "5. Длинные функции (> 60 строк) — top 10"

# Simple heuristic: find fn definitions and count lines to next fn or }
long_fns=""
for dir in "${EXISTING[@]}"; do
    while IFS= read -r file; do
        # Use awk to find functions and their lengths
        awk '
        /^[[:space:]]*(pub )?(async )?fn / {
            if (fn_name != "" && fn_lines > 60) {
                printf "  %s:%d %s (%d lines)\n", FILENAME, fn_start, fn_name, fn_lines
            }
            fn_name = $0
            gsub(/.*fn /, "", fn_name)
            gsub(/\(.*/, "", fn_name)
            fn_start = NR
            fn_lines = 0
            brace_count = 0
            in_fn = 1
        }
        in_fn { fn_lines++ }
        END {
            if (fn_name != "" && fn_lines > 60) {
                printf "  %s:%d %s (%d lines)\n", FILENAME, fn_start, fn_name, fn_lines
            }
        }
        ' "$file" 2>/dev/null
    done < <(find "$dir" -name "*.rs" -type f 2>/dev/null)
done | sort -t'(' -k2 -rn | head -10 > /tmp/rustok_long_fns.txt

if [[ -s /tmp/rustok_long_fns.txt ]]; then
    count=$(wc -l < /tmp/rustok_long_fns.txt)
    warn "$count function(s) exceed 60 lines:"
    cat /tmp/rustok_long_fns.txt
else
    pass "No functions exceed 60 lines"
fi
rm -f /tmp/rustok_long_fns.txt

# ═══════════════════════════════════════════
# DEPENDENCY ANTIPATTERNS
# ═══════════════════════════════════════════

# ─── 6. rustok-core не зависит от domain crates ───
header "6. Dependency: rustok-core independence"

if [[ -f "crates/rustok-core/Cargo.toml" ]]; then
    core_deps=$(grep -E 'rustok-(content|commerce|blog|forum|pages)' "crates/rustok-core/Cargo.toml" 2>/dev/null || true)
    if [[ -n "$core_deps" ]]; then
        fail "rustok-core depends on domain crates (circular dependency!):"
        echo "$core_deps"
    else
        pass "rustok-core doesn't depend on domain crates"
    fi
fi

# ─── 7. Domain crates не зависят друг от друга ───
header "7. Dependency: domain crate independence"

domain_crate_names=("rustok-content" "rustok-commerce" "rustok-blog" "rustok-forum" "rustok-pages")
for crate in "${domain_crate_names[@]}"; do
    cargo_toml="crates/$crate/Cargo.toml"
    if [[ -f "$cargo_toml" ]]; then
        other_domains=""
        for other in "${domain_crate_names[@]}"; do
            if [[ "$crate" != "$other" ]]; then
                if grep -q "$other" "$cargo_toml" 2>/dev/null; then
                    other_domains+=" $other"
                fi
            fi
        done
        if [[ -n "$other_domains" ]]; then
            warn "$crate depends on:$other_domains (should communicate via events)"
        else
            pass "$crate — independent from other domain crates"
        fi
    fi
done

# ─── 8. rustok-test-utils only in dev-dependencies ───
header "8. Dependency: test-utils only in dev-dependencies"

test_utils_in_deps=$(grep -rl 'rustok-test-utils' crates/*/Cargo.toml apps/*/Cargo.toml 2>/dev/null || true)
if [[ -n "$test_utils_in_deps" ]]; then
    for cargo_file in $test_utils_in_deps; do
        # Check if it's under [dev-dependencies] or [dependencies]
        in_dev=$(awk '/\[dev-dependencies\]/,/\[/' "$cargo_file" 2>/dev/null | grep "rustok-test-utils" || true)
        in_deps=$(awk '/\[dependencies\]/,/\[/' "$cargo_file" 2>/dev/null | grep "rustok-test-utils" || true)
        if [[ -n "$in_deps" && -z "$in_dev" ]]; then
            fail "$cargo_file — rustok-test-utils in [dependencies] (should be [dev-dependencies])"
        else
            pass "$cargo_file — rustok-test-utils in [dev-dependencies]"
        fi
    done
else
    pass "rustok-test-utils not referenced (or only in dev)"
fi

# ═══════════════════════════════════════════
# ERROR HANDLING
# ═══════════════════════════════════════════

# ─── 9. thiserror в domain crates ───
header "9. Error handling: thiserror in domain crates"

for crate in "${domain_crate_names[@]}"; do
    cargo_toml="crates/$crate/Cargo.toml"
    if [[ -f "$cargo_toml" ]]; then
        if grep -q "thiserror" "$cargo_toml" 2>/dev/null; then
            pass "$crate — uses thiserror"
        else
            warn "$crate — thiserror not in dependencies"
        fi
    fi
done

# ─── 10. anyhow в library crates (antipattern) ───
header "10. Error handling: anyhow in library crates (antipattern)"

for crate in "${domain_crate_names[@]}" "rustok-core" "rustok-rbac" "rustok-events"; do
    cargo_toml="crates/$crate/Cargo.toml"
    if [[ -f "$cargo_toml" ]]; then
        if grep -q "anyhow" "$cargo_toml" 2>/dev/null; then
            warn "$crate — uses anyhow (prefer thiserror for library crates)"
        else
            pass "$crate — no anyhow (good for library crate)"
        fi
    fi
done

# ─── 11. String-based status checks ───
header "11. String-based status/state checks (antipattern)"

string_status=$(grep -rn '"published"\|"draft"\|"archived"\|"active"\|"inactive"\|"pending"' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | grep -iE 'status\s*==\|state\s*==\|==\s*"' | grep -v "test\|// \|///\|migration\|assert" || true)
if [[ -n "$string_status" ]]; then
    count=$(echo "$string_status" | wc -l)
    warn "$count string-based status check(s) (should use enum):"
    echo "$string_status" | head -10
else
    pass "No string-based status checks (using enums)"
fi

# ═══════════════════════════════════════════
# OBSERVABILITY
# ═══════════════════════════════════════════

# ─── 12. #[instrument] на service methods ───
header "12. Observability: #[instrument] on service methods"

SERVICE_DIRS=()
for crate in "${PROD_RS_PATHS[@]}"; do
    svc_dir="$crate/services"
    [[ -d "$svc_dir" ]] && SERVICE_DIRS+=("$svc_dir")
    # Also check for service.rs files
    svc_file="$crate/service.rs"
    [[ -f "$svc_file" ]] && SERVICE_DIRS+=("$svc_file")
done

if [[ ${#SERVICE_DIRS[@]} -gt 0 ]]; then
    total_svc_fns=0
    instrumented_fns=0
    for svc in "${SERVICE_DIRS[@]}"; do
        while IFS= read -r file; do
            fns=$(grep -cn 'pub async fn\|pub fn' "$file" 2>/dev/null || echo "0")
            instrs=$(grep -cn '#\[instrument' "$file" 2>/dev/null || echo "0")
            total_svc_fns=$((total_svc_fns + fns))
            instrumented_fns=$((instrumented_fns + instrs))
        done < <(find "$svc" -name "*.rs" -type f 2>/dev/null)
    done

    if [[ $total_svc_fns -gt 0 ]]; then
        pct=$((instrumented_fns * 100 / total_svc_fns))
        if [[ $pct -ge 80 ]]; then
            pass "#[instrument] coverage: $instrumented_fns/$total_svc_fns service functions ($pct%)"
        elif [[ $pct -ge 50 ]]; then
            warn "#[instrument] coverage: $instrumented_fns/$total_svc_fns service functions ($pct%) — aim for 80%+"
        else
            warn "#[instrument] coverage: $instrumented_fns/$total_svc_fns service functions ($pct%) — LOW"
        fi
    fi
fi

# ─── 13. Structured logging (not string interpolation) ───
header "13. Observability: structured logging"

string_format_logs=$(grep -rn 'tracing::\|info!\|debug!\|warn!\|error!' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | grep -E 'format!\|&format' | grep -v "test\|// " || true)
if [[ -n "$string_format_logs" ]]; then
    count=$(echo "$string_format_logs" | wc -l)
    warn "$count log call(s) using format! (use structured fields instead):"
    echo "$string_format_logs" | head -10
else
    pass "No format! in tracing calls (using structured fields)"
fi

# ═══════════════════════════════════════════
# TYPE SAFETY
# ═══════════════════════════════════════════

# ─── 14. Newtype IDs ───
header "14. Type safety: Newtype IDs (not bare Uuid)"

bare_uuid_params=$(grep -rn 'Path<Uuid>\|Query.*Uuid>\|Json.*uuid::Uuid' "apps/server/src/controllers" --include="*.rs" 2>/dev/null | grep -v "test\|// " || true)
if [[ -n "$bare_uuid_params" ]]; then
    count=$(echo "$bare_uuid_params" | wc -l)
    warn "$count bare Uuid parameter(s) in controllers (should use TenantId, UserId, etc.):"
    echo "$bare_uuid_params" | head -10
else
    pass "No bare Uuid in controller parameters"
fi

# ─── 15. Function arity (> 5 args) ───
header "15. Code metrics: functions with > 5 arguments"

# Find function signatures with many commas (heuristic for argument count)
high_arity=""
for dir in "${EXISTING[@]}"; do
    while IFS= read -r file; do
        # Find fn lines, count commas in signature
        grep -n 'pub\s\+\(async\s\+\)\?fn ' "$file" 2>/dev/null | while IFS= read -r line; do
            lineno=$(echo "$line" | cut -d: -f1)
            fn_name=$(echo "$line" | grep -oP 'fn\s+\K\w+' || echo "unknown")
            # Get full signature (may span lines)
            sig=$(sed -n "${lineno},$((lineno + 5))p" "$file" 2>/dev/null | tr '\n' ' ' | sed 's/\s\+/ /g' || true)
            # Count commas between ( and )
            params=$(echo "$sig" | grep -oP '\(.*?\)' | head -1 || true)
            comma_count=$(echo "$params" | tr -cd ',' | wc -c)
            if [[ $comma_count -gt 5 ]]; then
                echo "  $file:$lineno $fn_name — $((comma_count + 1)) params"
            fi
        done
    done < <(find "$dir" -name "*.rs" -type f 2>/dev/null)
done > /tmp/rustok_high_arity.txt 2>/dev/null

if [[ -s /tmp/rustok_high_arity.txt ]]; then
    count=$(wc -l < /tmp/rustok_high_arity.txt)
    warn "$count function(s) with > 5 arguments (consider using param struct):"
    cat /tmp/rustok_high_arity.txt | head -10
else
    pass "No functions with > 5 arguments"
fi
rm -f /tmp/rustok_high_arity.txt

# ─── Summary ───
echo ""
echo -e "${BOLD}━━━ Code Quality Summary ━━━${NC}"
if [[ $ERRORS -eq 0 && $WARNINGS -eq 0 ]]; then
    echo -e "${GREEN}All checks passed!${NC}"
elif [[ $ERRORS -eq 0 ]]; then
    echo -e "${YELLOW}$WARNINGS warning(s) — manual review recommended${NC}"
else
    echo -e "${RED}$ERRORS error(s), $WARNINGS warning(s)${NC}"
fi
exit $ERRORS
