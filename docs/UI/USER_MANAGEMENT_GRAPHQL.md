# User Management GraphQL API

Complete GraphQL API documentation for user management operations in RusToK Admin UI.

---

## Overview

The User Management GraphQL API provides comprehensive CRUD operations for managing users within a tenant. All operations are tenant-isolated and protected by RBAC permissions.

---

## GraphQL Schema

### Queries

#### `user` - Get Single User

Retrieve a single user by ID.

```graphql
query User($id: ID!) {
  user(id: $id) {
    id
    email
    name
    role
    status
    createdAt
    displayName
    can(action: "users:update")
    tenantName
  }
}
```

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | UUID | Yes | User ID |

**Response:**
```json
{
  "data": {
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "john.doe@example.com",
      "name": "John Doe",
      "role": "ADMIN",
      "status": "ACTIVE",
      "createdAt": "2026-02-16T10:30:00Z",
      "displayName": "John Doe",
      "can": true,
      "tenantName": "Acme Corp"
    }
  }
}
```

**Required Permission:** `users:read`

---

#### `users` - List Users

Retrieve a paginated list of users with optional filtering and search.

```graphql
query Users(
  $search: String,
  $role: GqlUserRole,
  $status: GqlUserStatus,
  $first: Int = 20,
  $after: String
) {
  users(
    search: $search,
    filter: { role: $role, status: $status },
    pagination: { first: $first, after: $after }
  ) {
    edges {
      node {
        id
        email
        name
        role
        status
        createdAt
      }
      cursor
    }
    pageInfo {
      hasNextPage
      hasPreviousPage
      startCursor
      endCursor
      total
    }
  }
}
```

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `search` | String | No | Search by email or name |
| `role` | GqlUserRole | No | Filter by role (SUPER_ADMIN, ADMIN, MANAGER, CUSTOMER) |
| `status` | GqlUserStatus | No | Filter by status (ACTIVE, INACTIVE, BANNED) |
| `first` | Int | No | Number of items per page (default: 20) |
| `after` | String | No | Cursor for pagination |

**Response:**
```json
{
  "data": {
    "users": {
      "edges": [
        {
          "node": {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "email": "admin@example.com",
            "name": "Admin User",
            "role": "ADMIN",
            "status": "ACTIVE",
            "createdAt": "2026-02-16T10:00:00Z"
          },
          "cursor": "eyJvZmZzZXQiOjB9"
        }
      ],
      "pageInfo": {
        "hasNextPage": true,
        "hasPreviousPage": false,
        "startCursor": "eyJvZmZzZXQiOjB9",
        "endCursor": "eyJvZmZzZXQiOjE5fQ",
        "total": 150
      }
    }
  }
}
```

**Required Permission:** `users:list`

---

### Mutations

#### `createUser` - Create New User

Create a new user in the current tenant.

```graphql
mutation CreateUser($input: CreateUserInput!) {
  createUser(input: $input) {
    id
    email
    name
    role
    status
    createdAt
  }
}
```

**Input:**
```graphql
input CreateUserInput {
  email: String!
  password: String!
  name: String
  role: GqlUserRole
  status: GqlUserStatus
}
```

**Example Variables:**
```json
{
  "input": {
    "email": "new.user@example.com",
    "password": "SecurePass123!",
    "name": "New User",
    "role": "MANAGER",
    "status": "ACTIVE"
  }
}
```

**Response:**
```json
{
  "data": {
    "createUser": {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "email": "new.user@example.com",
      "name": "New User",
      "role": "MANAGER",
      "status": "ACTIVE",
      "createdAt": "2026-02-16T14:30:00Z"
    }
  }
}
```

**Required Permission:** `users:create` or `users:manage`

**Validation:**
- Email must be unique within tenant
- Password must meet security requirements
- Email is normalized to lowercase

---

#### `updateUser` - Update Existing User

Update an existing user's information.

```graphql
mutation UpdateUser($id: ID!, $input: UpdateUserInput!) {
  updateUser(id: $id, input: $input) {
    id
    email
    name
    role
    status
    createdAt
  }
}
```

**Input:**
```graphql
input UpdateUserInput {
  email: String
  password: String
  name: String
  role: GqlUserRole
  status: GqlUserStatus
}
```

**Example Variables:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "input": {
    "name": "Updated Name",
    "role": "ADMIN",
    "status": "ACTIVE"
  }
}
```

**Response:**
```json
{
  "data": {
    "updateUser": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "john.doe@example.com",
      "name": "Updated Name",
      "role": "ADMIN",
      "status": "ACTIVE",
      "createdAt": "2026-02-16T10:30:00Z"
    }
  }
}
```

**Required Permission:** `users:update` or `users:manage`

**Validation:**
- Email must be unique (if changed)
- Cannot change email to one used by another user
- Password is hashed if provided

---

#### `deleteUser` - Delete User

Permanently delete a user from the system.

```graphql
mutation DeleteUser($id: ID!) {
  deleteUser(id: $id)
}
```

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | UUID | Yes | User ID to delete |

**Example Variables:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response:**
```json
{
  "data": {
    "deleteUser": true
  }
}
```

**Required Permission:** `users:delete`

**Important Notes:**
- This is a **permanent deletion** - user data is removed from database
- For soft deletion (mark as inactive), use `disableUser` mutation instead
- Tenant isolation enforced - can only delete users from current tenant

---

#### `disableUser` - Soft Delete User

Mark a user as inactive (soft delete).

```graphql
mutation DisableUser($id: ID!) {
  disableUser(id: $id) {
    id
    email
    name
    status
  }
}
```

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | UUID | Yes | User ID to disable |

**Response:**
```json
{
  "data": {
    "disableUser": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "john.doe@example.com",
      "name": "John Doe",
      "status": "INACTIVE"
    }
  }
}
```

**Required Permission:** `users:manage`

---

## Types

### User

```graphql
type User {
  id: UUID!
  email: String!
  name: String
  role: String!
  status: String!
  createdAt: String!
  tenantId: UUID!
  displayName: String!
  can(action: String!): Boolean!
  tenantName: String
}
```

### Enums

#### GqlUserRole
```graphql
enum GqlUserRole {
  SUPER_ADMIN
  ADMIN
  MANAGER
  CUSTOMER
}
```

#### GqlUserStatus
```graphql
enum GqlUserStatus {
  ACTIVE
  INACTIVE
  BANNED
}
```

### Connection Types

#### UserConnection
```graphql
type UserConnection {
  edges: [UserEdge!]!
  pageInfo: PageInfo!
}
```

#### UserEdge
```graphql
type UserEdge {
  node: User!
  cursor: String!
}
```

#### PageInfo
```graphql
type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
  total: Int!
}
```

---

## Frontend Integration (Leptos)

### Using leptos-graphql

```rust
use leptos::*;
use leptos_graphql::*;

// List users with pagination
#[component]
fn UsersList() -> impl IntoView {
    let (search, set_search) = create_signal(String::new());
    let (role_filter, set_role_filter) = create_signal(None::<GqlUserRole>);
    
    let users_query = use_query(move || {
        let search = search.get();
        let role = role_filter.get();
        
        UsersQuery::build(UsersQueryVariables {
            search: if search.is_empty() { None } else { Some(search) },
            role,
            first: Some(20),
            after: None,
        })
    });
    
    view! {
        <div>
            <input
                type="text"
                placeholder="Search users..."
                on:input=move |ev| set_search.set(event_target_value(&ev))
            />
            
            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                {move || users_query.data.get().map(|data| {
                    view! {
                        <UsersTable users={data.users.edges} />
                        <Pagination page_info={data.users.page_info} />
                    }
                })}
            </Suspense>
        </div>
    }
}

// Create user mutation
#[component]
fn CreateUserForm() -> impl IntoView {
    let create_user = use_mutation(|input: CreateUserInput| {
        CreateUserMutation::build(CreateUserMutationVariables { input })
    });
    
    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let input = CreateUserInput {
            email: "user@example.com".to_string(),
            password: "SecurePass123!".to_string(),
            name: Some("New User".to_string()),
            role: Some(GqlUserRole::Manager),
            status: Some(GqlUserStatus::Active),
        };
        create_user.mutate(input);
    };
    
    view! {
        <form on:submit=on_submit>
            // Form fields...
            <button type="submit" disabled=create_user.loading>
                {move || if create_user.loading.get() { "Creating..." } else { "Create User" }}
            </button>
        </form>
    }
}

// Delete user with confirmation
#[component]
fn DeleteUserButton(user_id: String) -> impl IntoView {
    let delete_user = use_mutation(|id: String| {
        DeleteUserMutation::build(DeleteUserMutationVariables { id })
    });
    
    let on_delete = move |_| {
        if confirm("Are you sure you want to delete this user?") {
            delete_user.mutate(user_id.clone());
        }
    };
    
    view! {
        <button on:click=on_delete class="danger">
            "Delete User"
        </button>
    }
}
```

---

## Error Handling

### Common Error Codes

| Error | Code | Description |
|-------|------|-------------|
| User not found | `NOT_FOUND` | User ID does not exist in tenant |
| Email exists | `CONFLICT` | Email already used by another user |
| Permission denied | `FORBIDDEN` | User lacks required RBAC permission |
| Unauthorized | `UNAUTHENTICATED` | Missing or invalid authentication |
| Validation error | `BAD_USER_INPUT` | Invalid input data |

### Error Response Example

```json
{
  "errors": [
    {
      "message": "User with this email already exists",
      "extensions": {
        "code": "CONFLICT"
      }
    }
  ]
}
```

---

## RBAC Permissions

| Operation | Required Permission |
|-----------|---------------------|
| `user` | `users:read` |
| `users` | `users:list` |
| `createUser` | `users:create` or `users:manage` |
| `updateUser` | `users:update` or `users:manage` |
| `disableUser` | `users:manage` |
| `deleteUser` | `users:delete` |

---

## Security Considerations

1. **Tenant Isolation**: All queries are automatically filtered by current tenant
2. **RBAC Enforcement**: Every operation checks user permissions
3. **Email Uniqueness**: Emails are unique within tenant, case-insensitive
4. **Password Hashing**: Passwords are hashed with Argon2 before storage
5. **Audit Trail**: Consider implementing audit logging for user mutations

---

## Testing

### Manual Testing

Test at `http://localhost:5150/api/graphql`:

**Create user:**
```graphql
mutation {
  createUser(input: {
    email: "test@example.com"
    password: "TestPass123!"
    name: "Test User"
    role: MANAGER
    status: ACTIVE
  }) {
    id
    email
    role
  }
}
```

**List users:**
```graphql
query {
  users(pagination: { first: 10 }) {
    edges {
      node {
        id
        email
        name
        role
        status
      }
    }
    pageInfo {
      total
      hasNextPage
    }
  }
}
```

**Delete user:**
```graphql
mutation {
  deleteUser(id: "USER_UUID_HERE")
}
```

---

## Future Enhancements

- [ ] Bulk user operations (create, update, delete)
- [ ] User import from CSV/Excel
- [ ] Advanced filtering (date ranges, custom fields)
- [ ] User activity audit log
- [ ] Soft delete with restore functionality
- [ ] User profile image upload
- [ ] Email verification workflow
- [ ] Password reset functionality

---

*Last updated: 2026-02-16*
