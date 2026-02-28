#!/usr/bin/env bash
# RusTok — Верификация unsafe code patterns
# Фаза 19.1, 19.3: unwrap/expect/panic, blocking в async, std::fs в async
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

# Production code paths (excluding tests)
PROD_PATHS=(
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
for p in "${PROD_PATHS[@]}"; do
    [[ -d "$p" ]] && EXISTING+=("$p")
done

if [[ ${#EXISTING[@]} -eq 0 ]]; then
    echo -e "${YELLOW}No production code paths found. Skipping.${NC}"
    exit 0
fi

# Helper: filter out test files and test modules from grep output
filter_tests() {
    grep -v "_test\.rs" | grep -v "tests\.rs" | grep -v "test_utils" | grep -v "mod tests" | grep -v "#\[cfg(test)\]" | grep -v "#\[test\]" | grep -v "proptest" || true
}

# ─── 1. unwrap() в production коде ───
header "1. Поиск .unwrap() в production коде"

count=$(grep -rn '\.unwrap()' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | wc -l)
if [[ $count -eq 0 ]]; then
    pass "No .unwrap() calls in production code"
else
    warn "$count .unwrap() call(s) found — review each one:"
    grep -rn '\.unwrap()' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | head -20
    if [[ $count -gt 20 ]]; then
        echo "      ... and $((count - 20)) more"
    fi
fi

# ─── 2. expect() в production коде ───
header "2. Поиск .expect() в production коде"

count=$(grep -rn '\.expect(' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | wc -l)
if [[ $count -eq 0 ]]; then
    pass "No .expect() calls in production code"
else
    warn "$count .expect() call(s) found — review each (some may be justified):"
    grep -rn '\.expect(' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | head -20
    if [[ $count -gt 20 ]]; then
        echo "      ... and $((count - 20)) more"
    fi
fi

# ─── 3. panic!() в production коде ───
header "3. Поиск panic!() в production коде"

count=$(grep -rn 'panic!(' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | wc -l)
if [[ $count -eq 0 ]]; then
    pass "No panic!() in production code"
else
    fail "$count panic!() call(s) found:"
    grep -rn 'panic!(' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | head -20
fi

# ─── 4. todo!() / unimplemented!() в production коде ───
header "4. Поиск todo!() / unimplemented!() в production коде"

count=$(grep -rn 'todo!()\|unimplemented!()' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | wc -l)
if [[ $count -eq 0 ]]; then
    pass "No todo!()/unimplemented!() in production code"
else
    fail "$count todo!()/unimplemented!() call(s) found:"
    grep -rn 'todo!()\|unimplemented!()' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | head -20
fi

# ─── 5. std::thread::sleep в async коде ───
header "5. Поиск std::thread::sleep в async коде"

count=$(grep -rn 'std::thread::sleep\|thread::sleep' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | wc -l)
if [[ $count -eq 0 ]]; then
    pass "No std::thread::sleep in production code"
else
    fail "$count blocking sleep call(s) found (should use tokio::time::sleep):"
    grep -rn 'std::thread::sleep\|thread::sleep' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests
fi

# ─── 6. std::fs:: в async коде ───
header "6. Поиск std::fs:: в async коде (should be tokio::fs::)"

count=$(grep -rn 'std::fs::' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | wc -l)
if [[ $count -eq 0 ]]; then
    pass "No std::fs:: in production code"
else
    warn "$count std::fs:: usage(s) found (should use tokio::fs:: in async context):"
    grep -rn 'std::fs::' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | head -20
fi

# ─── 7. block_on() внутри async context ───
header "7. Поиск block_on() внутри async context"

count=$(grep -rn 'block_on\|block_in_place' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | wc -l)
if [[ $count -eq 0 ]]; then
    pass "No block_on()/block_in_place() in production code"
else
    warn "$count block_on/block_in_place call(s) found:"
    grep -rn 'block_on\|block_in_place' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | head -10
fi

# ─── 8. println!/eprintln! вместо tracing:: ───
header "8. Поиск println!/eprintln! в production коде (should use tracing::)"

count=$(grep -rn 'println!\|eprintln!' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | wc -l)
if [[ $count -eq 0 ]]; then
    pass "No println!/eprintln! in production code"
else
    warn "$count println!/eprintln! call(s) found (should use tracing::):"
    grep -rn 'println!\|eprintln!' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | head -20
fi

# ─── 9. unreachable!() без обоснования ───
header "9. Поиск unreachable!() в production коде"

count=$(grep -rn 'unreachable!(' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | wc -l)
if [[ $count -eq 0 ]]; then
    pass "No unreachable!() in production code"
else
    warn "$count unreachable!() call(s) — verify each is justified:"
    grep -rn 'unreachable!(' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | head -10
fi

# ─── 10. static / lazy_static! / once_cell::Lazy (should use AppContext) ───
header "10. Поиск global state: static / lazy_static / once_cell::Lazy"

static_state=$(grep -rn 'static\s\+mut\|lazy_static!\|once_cell::sync::Lazy\|static\s\+ref' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | filter_tests | grep -v "const\|type\|str\|// " || true)
if [[ -n "$static_state" ]]; then
    count=$(echo "$static_state" | wc -l)
    warn "$count global state declaration(s) — should use AppContext.shared_store:"
    echo "$static_state" | head -10
else
    pass "No global mutable state (lazy_static/once_cell)"
fi

# ─── 11. Unsafe fallback defaults for secrets ───
header "11. Unsafe fallback defaults for secrets"

unsafe_fallback=$(grep -rn 'unwrap_or\|unwrap_or_else\|unwrap_or_default' "${EXISTING[@]}" --include="*.rs" 2>/dev/null | grep -iE 'secret\|password\|jwt\|token\|key\|api_key' | filter_tests || true)
if [[ -n "$unsafe_fallback" ]]; then
    count=$(echo "$unsafe_fallback" | wc -l)
    fail "$count unsafe fallback(s) for secrets (should fail loudly, not use defaults):"
    echo "$unsafe_fallback" | head -10
else
    pass "No unsafe fallback defaults for secrets"
fi

# ─── Summary ───
echo ""
echo -e "${BOLD}━━━ Unsafe Code Summary ━━━${NC}"
if [[ $ERRORS -eq 0 && $WARNINGS -eq 0 ]]; then
    echo -e "${GREEN}All checks passed!${NC}"
elif [[ $ERRORS -eq 0 ]]; then
    echo -e "${YELLOW}$WARNINGS warning(s) — manual review recommended${NC}"
else
    echo -e "${RED}$ERRORS error(s), $WARNINGS warning(s)${NC}"
fi
exit $ERRORS
