#!/usr/bin/env bash
# RusTok — Верификация событийной системы
# Фаза 6 + 19.1: publish_in_tx, tenant_id в events, handler quality
set -euo pipefail

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
fail()   { echo -e "  ${RED}✗${NC} $1"; ((ERRORS++)); }
warn()   { echo -e "  ${YELLOW}!${NC} $1"; ((WARNINGS++)); }

DOMAIN_CRATES=(
    "crates/rustok-content/src"
    "crates/rustok-commerce/src"
    "crates/rustok-blog/src"
    "crates/rustok-forum/src"
    "crates/rustok-pages/src"
)

EVENTS_CRATE="crates/rustok-events/src"
CORE_CRATE="crates/rustok-core/src"
OUTBOX_CRATE="crates/rustok-outbox/src"

EXISTING_DOMAIN=()
for crate in "${DOMAIN_CRATES[@]}"; do
    [[ -d "$crate" ]] && EXISTING_DOMAIN+=("$crate")
done

# ─── 1. publish() без _in_tx ───
header "1. Поиск publish() без _in_tx в domain services"

if [[ ${#EXISTING_DOMAIN[@]} -gt 0 ]]; then
    # Find publish calls that are NOT publish_in_tx
    unsafe_publish=$(grep -rn '\.publish(' "${EXISTING_DOMAIN[@]}" --include="*.rs" 2>/dev/null | grep -v "publish_in_tx\|test\|// " || true)
    if [[ -n "$unsafe_publish" ]]; then
        count=$(echo "$unsafe_publish" | wc -l)
        fail "$count publish() call(s) without _in_tx (data loss risk):"
        echo "$unsafe_publish" | head -20
    else
        pass "All publish calls use publish_in_tx"
    fi

    # Verify publish_in_tx is actually used
    safe_publish=$(grep -rn 'publish_in_tx' "${EXISTING_DOMAIN[@]}" --include="*.rs" 2>/dev/null | grep -v "test\|// " || true)
    if [[ -n "$safe_publish" ]]; then
        count=$(echo "$safe_publish" | wc -l)
        pass "$count publish_in_tx call(s) found"
    else
        warn "No publish_in_tx calls found in domain crates"
    fi
else
    warn "No domain crates found"
fi

# ─── 2. DomainEvent variants have tenant_id ───
header "2. DomainEvent variants: tenant_id field"

event_files=$(find "$CORE_CRATE" "$EVENTS_CRATE" -name "*.rs" 2>/dev/null | sort || true)
if [[ -n "$event_files" ]]; then
    # Find DomainEvent enum or event structs
    event_defs=$(grep -rn 'struct.*Event\|enum DomainEvent\|enum.*Event' $event_files 2>/dev/null | grep -v "test\|Handler\|Bus\|Publisher" || true)
    if [[ -n "$event_defs" ]]; then
        total_events=0
        events_with_tenant=0

        while IFS= read -r line; do
            file=$(echo "$line" | cut -d: -f1)
            lineno=$(echo "$line" | cut -d: -f2)
            name=$(echo "$line" | grep -oP '(struct|enum)\s+\K\w+' || echo "unknown")

            # Check if it's an enum or struct
            if echo "$line" | grep -q "enum"; then
                # For enums, check that variants contain tenant_id
                enum_body=$(sed -n "${lineno},$((lineno + 100))p" "$file" 2>/dev/null | sed '/^}/q' || true)
                if echo "$enum_body" | grep -qi "tenant_id"; then
                    pass "$name enum — contains tenant_id in variants"
                else
                    warn "$name enum — tenant_id not found in variants"
                fi
            else
                ((total_events++))
                # For structs, check tenant_id field
                struct_body=$(sed -n "${lineno},$((lineno + 20))p" "$file" 2>/dev/null || true)
                if echo "$struct_body" | grep -qi "tenant_id"; then
                    pass "$name — has tenant_id field"
                    ((events_with_tenant++))
                else
                    warn "$name — no tenant_id field ($file:$lineno)"
                fi
            fi
        done <<< "$event_defs"

        if [[ $total_events -gt 0 ]]; then
            echo -e "\n  Event structs with tenant_id: $events_with_tenant/$total_events"
        fi
    else
        warn "No event definitions found"
    fi
else
    warn "Event source files not found"
fi

# ─── 3. Event handlers exist ───
header "3. Event handlers registration"

handler_files=$(grep -rl "EventHandler\|impl.*Handler\|handle_event\|on_event" "apps/server/src" "${EXISTING_DOMAIN[@]}" --include="*.rs" 2>/dev/null || true)
if [[ -n "$handler_files" ]]; then
    count=$(echo "$handler_files" | wc -l)
    pass "$count file(s) with event handler implementations"
else
    warn "No event handler implementations found"
fi

# ─── 4. Outbox pattern ───
header "4. Outbox pattern implementation"

if [[ -d "$OUTBOX_CRATE" ]]; then
    pass "rustok-outbox crate exists"

    if grep -rq "OutboxMessage\|outbox" "$OUTBOX_CRATE" --include="*.rs" 2>/dev/null; then
        pass "OutboxMessage type defined"
    else
        warn "OutboxMessage not found in outbox crate"
    fi

    # Check if outbox is used in domain crates
    outbox_usage=$(grep -rl "outbox\|Outbox" "${EXISTING_DOMAIN[@]}" --include="*.rs" 2>/dev/null || true)
    if [[ -n "$outbox_usage" ]]; then
        pass "Outbox referenced in domain crates"
    else
        warn "Outbox not referenced in domain crates"
    fi
else
    warn "rustok-outbox crate not found"
fi

# ─── 5. DLQ (Dead Letter Queue) ───
header "5. Dead Letter Queue (DLQ)"

dlq_refs=$(grep -rl "dlq\|dead_letter\|DeadLetter\|DLQ" "apps/server/src" "$CORE_CRATE" "$EVENTS_CRATE" --include="*.rs" 2>/dev/null || true)
if [[ -n "$dlq_refs" ]]; then
    count=$(echo "$dlq_refs" | wc -l)
    pass "DLQ referenced in $count file(s)"
else
    warn "No DLQ implementation found"
fi

# ─── 6. Event versioning ───
header "6. Event versioning"

version_refs=$(grep -rn 'version\|Version\|schema_version\|event_version' "$CORE_CRATE" "$EVENTS_CRATE" --include="*.rs" 2>/dev/null | grep -iE "event.*version\|version.*event" || true)
if [[ -n "$version_refs" ]]; then
    pass "Event versioning found"
else
    warn "No event versioning found"
fi

# ─── 7. Idempotency in handlers ───
header "7. Idempotency markers in event handlers"

idempotency_refs=$(grep -rn 'idempoten\|dedup\|already_processed\|processed_events' "apps/server/src" "${EXISTING_DOMAIN[@]}" --include="*.rs" 2>/dev/null || true)
if [[ -n "$idempotency_refs" ]]; then
    pass "Idempotency references found"
else
    warn "No idempotency markers found in event handlers"
fi

# ─── Summary ───
echo ""
echo -e "${BOLD}━━━ Events System Summary ━━━${NC}"
if [[ $ERRORS -eq 0 && $WARNINGS -eq 0 ]]; then
    echo -e "${GREEN}All checks passed!${NC}"
elif [[ $ERRORS -eq 0 ]]; then
    echo -e "${YELLOW}$WARNINGS warning(s) — manual review recommended${NC}"
else
    echo -e "${RED}$ERRORS error(s), $WARNINGS warning(s)${NC}"
fi
exit $ERRORS
