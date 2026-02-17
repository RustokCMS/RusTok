# Tenancy

RusToK is multi-tenant by design. Tenant resolution and caching are enforced at the platform layer.

## Key elements

- Tenant resolution middleware in the server.
- Tenant-aware caching and stampede protection.
- Tenant-specific migrations and data separation.

## References

- [Tenant resolver v2 migration](../TENANT_RESOLVER_V2_MIGRATION.md)
- [Tenant cache v2 migration](../TENANT_CACHE_V2_MIGRATION.md)
- [Tenant cache stampede protection](../tenant-cache-stampede-protection.md)
