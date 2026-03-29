#!/usr/bin/env bash
# RusTok вЂ” Р’РµСЂРёС„РёРєР°С†РёСЏ Р±РµР·РѕРїР°СЃРЅРѕСЃС‚Рё
# Р¤Р°Р·Р° 18: password hashing, security headers, SSRF, secrets, JWT
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
pass()   { echo -e "  ${GREEN}вњ“${NC} $1"; }
fail()   { echo -e "  ${RED}вњ—${NC} $1"; ERRORS=$((ERRORS + 1)); }
warn()   { echo -e "  ${YELLOW}!${NC} $1"; WARNINGS=$((WARNINGS + 1)); }

SERVER_SRC="apps/server/src"
CORE_SRC="crates/rustok-core/src"

# в”Ђв”Ђв”Ђ 1. Password hashing: Argon2 в”Ђв”Ђв”Ђ
header "1. Password hashing: Argon2 (not MD5/SHA256/bcrypt)"

if grep -rq "argon2\|Argon2" "$SERVER_SRC" "$CORE_SRC" --include="*.rs" 2>/dev/null; then
    pass "Argon2 password hashing found"
else
    fail "Argon2 not found вЂ” check password hashing algorithm"
fi

# Check for weak hashing algorithms
weak_hash=$(grep -rn 'md5\|sha256\|sha1\|bcrypt' "$SERVER_SRC" "$CORE_SRC" --include="*.rs" 2>/dev/null | grep -iE 'hash.*password\|password.*hash' | grep -v "test\|// \|///\|argon" || true)
if [[ -n "$weak_hash" ]]; then
    fail "Weak password hashing algorithm detected:"
    echo "$weak_hash"
else
    pass "No weak password hashing (MD5/SHA256/bcrypt for passwords)"
fi

# в”Ђв”Ђв”Ђ 2. JWT secret: env var, no fallback в”Ђв”Ђв”Ђ
header "2. JWT secret configuration"

jwt_refs=$(grep -rn 'jwt\|JWT' "$SERVER_SRC" "$CORE_SRC" --include="*.rs" 2>/dev/null | grep -iE 'secret\|key' | grep -v "test\|// \|///\|pub struct\|pub enum" || true)
if [[ -n "$jwt_refs" ]]; then
    # Check for env::var usage
    if echo "$jwt_refs" | grep -q "env::var\|env!\|std::env"; then
        pass "JWT secret loaded from env var"
    else
        warn "JWT secret may not be from env var вЂ” review:"
        echo "$jwt_refs" | head -5
    fi

    # Check for fallback/default
    jwt_fallback=$(echo "$jwt_refs" | grep -iE 'unwrap_or\|default\|"secret"\|"key"' || true)
    if [[ -n "$jwt_fallback" ]]; then
        fail "JWT secret has unsafe fallback/default:"
        echo "$jwt_fallback"
    else
        pass "No unsafe fallback for JWT secret"
    fi
fi

# в”Ђв”Ђв”Ђ 3. Token invalidation on password change в”Ђв”Ђв”Ђ
header "3. Token invalidation on password change"

# Check if password change logic invalidates existing tokens/sessions
password_change_files=$(grep -rl "change_password\|reset_password\|update_password" "$SERVER_SRC" "$CORE_SRC" --include="*.rs" 2>/dev/null || true)
if [[ -n "$password_change_files" ]]; then
    token_invalidation=false
    for file in $password_change_files; do
        if grep -q "invalidate\|revoke\|delete.*session\|clear.*token\|remove.*session" "$file" 2>/dev/null; then
            token_invalidation=true
            pass "Token/session invalidation found in $file"
            break
        fi
    done
    if ! $token_invalidation; then
        warn "Password change found but no token invalidation logic detected"
    fi
else
    warn "No password change implementation found"
fi

# в”Ђв”Ђв”Ђ 4. Security headers middleware в”Ђв”Ђв”Ђ
header "4. Security headers (CSP, X-Frame-Options, HSTS)"

security_headers=0

if grep -rq "Content-Security-Policy\|content_security_policy\|csp" "$SERVER_SRC" --include="*.rs" 2>/dev/null; then
    pass "Content-Security-Policy header configured"
    security_headers=$((security_headers + 1))
else
    warn "Content-Security-Policy header not found"
fi

if grep -rq "X-Frame-Options\|x_frame_options\|frame_options" "$SERVER_SRC" --include="*.rs" 2>/dev/null; then
    pass "X-Frame-Options header configured"
    security_headers=$((security_headers + 1))
else
    warn "X-Frame-Options header not found"
fi

if grep -rq "Strict-Transport-Security\|hsts\|HSTS" "$SERVER_SRC" --include="*.rs" 2>/dev/null; then
    pass "HSTS header configured"
    security_headers=$((security_headers + 1))
else
    warn "HSTS header not found"
fi

if grep -rq "X-Content-Type-Options\|x_content_type_options\|nosniff" "$SERVER_SRC" --include="*.rs" 2>/dev/null; then
    pass "X-Content-Type-Options header configured"
    security_headers=$((security_headers + 1))
else
    warn "X-Content-Type-Options header not found"
fi

echo -e "\n  Security headers: $security_headers/4 configured"

# в”Ђв”Ђв”Ђ 5. CORS configuration в”Ђв”Ђв”Ђ
header "5. CORS configuration"

cors_refs=$(grep -rn "CorsLayer\|cors\|Cors\|Access-Control" "$SERVER_SRC" --include="*.rs" 2>/dev/null || true)
if [[ -n "$cors_refs" ]]; then
    pass "CORS configuration found"
    # Check for wildcard origin
    if echo "$cors_refs" | grep -q 'any()\|"\*"\|AllowOrigin::any'; then
        warn "CORS allows ANY origin вЂ” restrict in production"
    else
        pass "CORS is not wildcard (restricted origins)"
    fi
else
    warn "No CORS configuration found"
fi

# в”Ђв”Ђв”Ђ 6. SSRF protection в”Ђв”Ђв”Ђ
header "6. SSRF protection (external HTTP requests)"

# Check for HTTP client usage without URL validation
http_clients=$(grep -rn 'reqwest::Client\|reqwest::get\|hyper::Client\|http_client' "$SERVER_SRC" "crates/rustok-core/src" "crates/alloy/src" --include="*.rs" 2>/dev/null | grep -v "test\|// " || true)
if [[ -n "$http_clients" ]]; then
    url_validation=$(grep -rn 'allowlist\|whitelist\|allowed_hosts\|validate_url\|is_safe_url' "$SERVER_SRC" "crates/rustok-core/src" "crates/alloy/src" --include="*.rs" 2>/dev/null || true)
    if [[ -n "$url_validation" ]]; then
        pass "URL validation/allowlist found for external requests"
    else
        warn "External HTTP client used but no URL allowlist/validation found (SSRF risk)"
    fi
else
    pass "No external HTTP client in application code (or using middleware)"
fi

# в”Ђв”Ђв”Ђ 7. Sensitive data in memory (zeroize) в”Ђв”Ђв”Ђ
header "7. Sensitive data: zeroize"

if grep -rq "zeroize\|Zeroize" "crates" "apps" --include="*.rs" --include="Cargo.toml" 2>/dev/null; then
    pass "zeroize dependency/usage found"
else
    warn "zeroize not found вЂ” sensitive data (passwords, keys) may persist in memory"
fi

# в”Ђв”Ђв”Ђ 8. .env files and .gitignore в”Ђв”Ђв”Ђ
header "8. Secrets management"

# Check .gitignore for .env
if [[ -f ".gitignore" ]]; then
    if grep -q "\.env" ".gitignore" 2>/dev/null; then
        pass ".gitignore excludes .env files"
    else
        fail ".gitignore does NOT exclude .env files"
    fi
fi

# Check for example env file
if [[ -f ".env.dev.example" || -f ".env.example" ]]; then
    pass "Example .env file exists"
    # Check it doesn't contain real secrets
    real_secrets=$(grep -iE '=.{20,}' .env*.example 2>/dev/null | grep -v "placeholder\|change_me\|your_\|xxx\|example\|TODO" || true)
    if [[ -n "$real_secrets" ]]; then
        warn "Example .env may contain real secrets (long values):"
        echo "$real_secrets" | head -5
    else
        pass "Example .env contains only placeholders"
    fi
else
    warn "No example .env file found"
fi

# в”Ђв”Ђв”Ђ 9. SQL injection patterns в”Ђв”Ђв”Ђ
header "9. SQL injection prevention"

# Check for raw SQL execution with string formatting
raw_sql=$(grep -rn 'execute_unprepared\|raw_sql\|SqlxRawSql\|query_as!\|sqlx::query!' "$SERVER_SRC" "crates" --include="*.rs" 2>/dev/null | grep -v "test\|migration\|// " || true)
if [[ -n "$raw_sql" ]]; then
    count=$(echo "$raw_sql" | wc -l)
    warn "$count raw SQL usage(s) found вЂ” verify parameterization:"
    echo "$raw_sql" | head -10
else
    pass "No raw SQL execution (using SeaORM query builder)"
fi

# в”Ђв”Ђв”Ђ 10. Rate limiting on sensitive endpoints в”Ђв”Ђв”Ђ
header "10. Rate limiting on sensitive endpoints"

if [[ -f "$SERVER_SRC/middleware/rate_limit.rs" ]]; then
    pass "Rate limiting middleware exists"
    rl_size=$(wc -l < "$SERVER_SRC/middleware/rate_limit.rs" 2>/dev/null || echo "0")
    echo -e "    Rate limit middleware: $rl_size lines"

    # Check if it covers auth endpoints specifically
    rl_auth=$(grep -n "auth\|login\|register\|password" "$SERVER_SRC/middleware/rate_limit.rs" 2>/dev/null || true)
    if [[ -n "$rl_auth" ]]; then
        pass "Rate limiting references auth endpoints"
    else
        warn "Rate limiting doesn't specifically target auth endpoints"
    fi
else
    fail "Rate limiting middleware not found"
fi

# в”Ђв”Ђв”Ђ Summary в”Ђв”Ђв”Ђ
echo ""
echo -e "${BOLD}в”Ѓв”Ѓв”Ѓ Security Summary в”Ѓв”Ѓв”Ѓ${NC}"
if [[ $ERRORS -eq 0 && $WARNINGS -eq 0 ]]; then
    echo -e "${GREEN}All checks passed!${NC}"
elif [[ $ERRORS -eq 0 ]]; then
    echo -e "${YELLOW}$WARNINGS warning(s) вЂ” manual review recommended${NC}"
else
    echo -e "${RED}$ERRORS error(s), $WARNINGS warning(s)${NC}"
fi
exit $ERRORS

