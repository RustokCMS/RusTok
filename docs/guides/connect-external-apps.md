# How to Connect External Apps (OAuth2)

RusTok supports connecting external applications, mobile apps, and machine-to-machine integrations using standard OAuth2. 

## App Types

When creating an app, you can choose from the following types:
- **ThirdParty**: Standard third-party apps that require explicit user consent.
- **FirstParty**: Internal apps (like Admin or Storefront) that do not require explicit user consent.
- **Mobile**: Native mobile apps (uses PKCE).
- **Service**: Machine-to-Machine token integrations.

## 1. Creating an App via Admin UI
You can manage your OAuth apps directly via the admin panel (both Leptos and Next.js):
1. Navigate to **App Connections** on the left sidebar.
2. Click **Create New App**.
3. Fill in the App Name, Slug, Description, and select the App Type.
4. Note your **Client Secret** when it is displayed — you will not be able to see it again!

## 2. Bootstrapping an App for Local Development (CLI)
If you are developing locally and want to quickly bootstrap a test application (e.g., Postman) without using the UI, you can use the built-in Loco CLI task:

```bash
cd apps/server
cargo loco task create_oauth_app name="My Postman Client" slug="postman-dev"
```

This command will output:
```text
==================================================
✅ OAuth Application created successfully!
Name:           My Postman Client
Type:           ThirdParty
Client ID:      [UUID]
Client Secret:  sk_live_...
==================================================
```

## 3. Implementing the OAuth2 Flow
Once you have your `client_id` and `client_secret`, you can implement standard OAuth2 Authorization Code flow (with or without PKCE):
1. Redirect user to `/api/oauth/authorize?client_id=...&response_type=code`
2. User authenticates and grants consent (if ThirdParty).
3. The server redirects back to your `redirect_uri` with a `code`.
4. Exchange the `code` for an access token via `POST /api/oauth/token`.

For detailed API definitions, see the core GraphQL API or our internal documentation map.
