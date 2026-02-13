# RBAC Enforcement in RusToK

This document describes the Role-Based Access Control (RBAC) enforcement system in RusToK.

## Overview

RusToK implements a comprehensive RBAC system that controls access to resources based on user roles. The system consists of:

1. **Resources** - The entities being protected (Products, Orders, Nodes, etc.)
2. **Actions** - Operations that can be performed (Create, Read, Update, Delete, List, etc.)
3. **Permissions** - Combinations of Resource + Action (e.g., `products:create`)
4. **Roles** - Collections of permissions assigned to users
5. **Enforcement** - Middleware and extractors that verify permissions

## User Roles

### SuperAdmin
- Full access to all resources and actions
- Can manage tenants, modules, and system-level settings
- Typically only 1-2 users per deployment

### Admin
- Full access to tenant-specific resources
- Can manage users, products, orders, content, etc.
- Cannot manage tenants or system modules
- Typically several per tenant

### Manager
- Can manage commerce (products, orders, inventory)
- Can create and manage content (posts, nodes, media)
- Can view customers and analytics
- Cannot manage users or system settings

### Customer
- Can view products and content
- Can create and view their own orders
- Can read posts and nodes
- Can create comments
- Limited write access

## Resources

The following resources are defined in the system:

- `users` - User accounts
- `tenants` - Tenant/organization data
- `modules` - System modules
- `settings` - System and tenant settings
- `products` - E-commerce products
- `categories` - Product categories
- `orders` - Customer orders
- `customers` - Customer data
- `inventory` - Stock management
- `discounts` - Discount codes and promotions
- `posts` - Blog posts
- `pages` - Static pages
- `nodes` - Content nodes (generic content)
- `media` - Media files and assets
- `comments` - User comments
- `analytics` - Analytics and reports
- `logs` - System logs
- `webhooks` - Webhook integrations

## Actions

- `create` - Create new resource
- `read` - Read single resource
- `update` - Update existing resource
- `delete` - Delete resource
- `list` - List/search resources
- `export` - Export data
- `import` - Import data
- `manage` - All actions on resource (wildcard)

## Permission Enforcement

### Method 1: Permission Extractors (Recommended)

The recommended way to enforce permissions is using custom extractors in your handlers:

```rust
use crate::extractors::rbac::{RequireNodesCreate, check_permission};
use rustok_core::Permission;

// Using pre-defined extractor
pub async fn create_node(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequireNodesCreate(user): RequireNodesCreate,  // ✅ Permission checked here
    Json(input): Json<CreateNodeInput>,
) -> Result<Json<NodeResponse>> {
    // User is guaranteed to have NODES_CREATE permission
    let service = NodeService::new(ctx.db.clone(), event_bus);
    let node = service
        .create_node(tenant.id, user.security_context(), input)
        .await?;
    Ok(Json(node))
}

// Using inline permission check
pub async fn some_handler(
    user: CurrentUser,
    // other extractors...
) -> Result<Response> {
    check_permission(&user, Permission::PRODUCTS_UPDATE)?;
    // Continue with handler logic
    // ...
}
```

### Method 2: Inline Permission Checks

For more complex scenarios, you can check permissions inline:

```rust
use crate::extractors::rbac::{check_permission, check_any_permission, check_all_permissions};
use rustok_core::Permission;

pub async fn complex_handler(
    user: CurrentUser,
    // ...
) -> Result<Response> {
    // Check single permission
    check_permission(&user, Permission::PRODUCTS_UPDATE)?;
    
    // Check if user has ANY of these permissions
    check_any_permission(&user, &[
        Permission::PRODUCTS_UPDATE,
        Permission::PRODUCTS_MANAGE,
    ])?;
    
    // Check if user has ALL of these permissions
    check_all_permissions(&user, &[
        Permission::PRODUCTS_READ,
        Permission::INVENTORY_UPDATE,
    ])?;
    
    // Continue...
}
```

### Method 3: Using SecurityContext in Services

Services receive a `SecurityContext` which includes role information. Services can use this to apply scope-based filtering:

```rust
impl NodeService {
    pub async fn list_nodes(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        filter: ListNodesFilter,
    ) -> Result<(Vec<NodeListItem>, i64)> {
        // Check permission scope
        let scope = security.get_scope(Resource::Nodes, Action::List);
        
        match scope {
            PermissionScope::All => {
                // Return all nodes for this tenant
            }
            PermissionScope::Own => {
                // Only return nodes authored by this user
                // filter.author_id = Some(security.user_id);
            }
            PermissionScope::None => {
                return Err(ContentError::PermissionDenied);
            }
        }
        
        // Execute query...
    }
}
```

## Available Permission Extractors

The following extractors are pre-defined and ready to use:

### Content
- `RequireNodesCreate`
- `RequireNodesRead`
- `RequireNodesUpdate`
- `RequireNodesDelete`
- `RequireNodesList`
- `RequirePostsCreate`
- `RequirePostsRead`
- `RequirePostsUpdate`
- `RequirePostsDelete`
- `RequirePostsList`

### Commerce
- `RequireProductsCreate`
- `RequireProductsRead`
- `RequireProductsUpdate`
- `RequireProductsDelete`
- `RequireProductsList`
- `RequireOrdersCreate`
- `RequireOrdersRead`
- `RequireOrdersUpdate`
- `RequireOrdersDelete`
- `RequireOrdersList`

### Administration
- `RequireUsersCreate`
- `RequireUsersRead`
- `RequireUsersUpdate`
- `RequireUsersDelete`
- `RequireUsersList`
- `RequireSettingsRead`
- `RequireSettingsUpdate`
- `RequireAnalyticsRead`
- `RequireAnalyticsExport`

## Creating Custom Permission Extractors

If you need a custom extractor, use the `define_permission_extractor!` macro:

```rust
define_permission_extractor!(RequireCustomAction, rustok_core::Permission::new(
    Resource::YourResource,
    Action::YourAction
));

// Then use in handler:
pub async fn handler(
    RequireCustomAction(user): RequireCustomAction,
    // ...
) -> Result<Response> {
    // User has permission
}
```

## Permission Scope

Some permissions have **scope** limitations:

- **PermissionScope::All** - Access to all resources of this type in the tenant
- **PermissionScope::Own** - Access only to resources owned by the user
- **PermissionScope::None** - No access

Example scopes:
- Customers have `Own` scope for Orders (can only see their own orders)
- Customers have `Own` scope for Comments updates/deletes
- Managers have `All` scope for Products (can manage all products)

## Testing Permission Enforcement

```rust
#[tokio::test]
async fn test_permission_denied() {
    let customer_user = create_test_user(UserRole::Customer);
    
    let result = check_permission(&customer_user, Permission::PRODUCTS_DELETE);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().0, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_permission_allowed() {
    let admin_user = create_test_user(UserRole::Admin);
    
    let result = check_permission(&admin_user, Permission::PRODUCTS_DELETE);
    
    assert!(result.is_ok());
}
```

## Best Practices

1. **Use extractors when possible** - They provide compile-time safety and clear intent
2. **Check permissions early** - Fail fast if user doesn't have permission
3. **Use scope for filtering** - Don't rely on permission checks alone; filter data by scope
4. **Document required permissions** - Add `@security` tags to OpenAPI docs
5. **Test negative cases** - Always test that unauthorized users are denied
6. **Avoid `unwrap()`** - Handle all permission check results explicitly

## Migration Checklist

To add RBAC enforcement to existing endpoints:

- [ ] Identify the resource and action for the endpoint
- [ ] Choose appropriate permission constant or create new one
- [ ] Add permission extractor to handler signature OR use inline check
- [ ] Update OpenAPI documentation with security requirements
- [ ] Add tests for both authorized and unauthorized access
- [ ] Update service methods to respect permission scope
- [ ] Document permission requirements in endpoint docstring

## Example: Complete Protected Endpoint

```rust
/// Create a new product
///
/// Requires: `products:create` permission
/// Roles: Admin, Manager
#[utoipa::path(
    post,
    path = "/api/products",
    tag = "commerce",
    request_body = CreateProductInput,
    responses(
        (status = 201, description = "Product created", body = ProductResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - insufficient permissions")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_product(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequireProductsCreate(user): RequireProductsCreate,
    Json(input): Json<CreateProductInput>,
) -> Result<Json<ProductResponse>> {
    let service = CatalogService::new(ctx.db.clone(), event_bus);
    
    let product = service
        .create_product(tenant.id, user.security_context(), input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    
    Ok(Json(product))
}
```

## See Also

- `crates/rustok-core/src/permissions.rs` - Permission definitions
- `crates/rustok-core/src/rbac.rs` - RBAC implementation
- `apps/server/src/extractors/rbac.rs` - Permission extractors
- `apps/server/src/extractors/auth.rs` - Authentication
