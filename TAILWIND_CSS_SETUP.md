# Tailwind CSS Setup for RusToK Leptos Applications

## Overview

RusToK uses **Tailwind CSS CLI** (official tool) for styling Leptos applications (admin and storefront).

We switched from `tailwind-rs` Rust crate to the official Tailwind CLI to avoid compilation issues with outdated `parcel_css` dependencies.

## Prerequisites

- Node.js (v18+) - for Tailwind CLI
- npm or npx

## Installation

Tailwind CSS is already installed as a dev dependency:

```bash
npm install
```

## Configuration Files

### Admin Panel (`apps/admin`)
- **Config**: `apps/admin/tailwind.config.js`
- **Input CSS**: `apps/admin/input.css`
- **Output CSS**: `apps/admin/dist/output.css` (generated, ignored by git)

### Storefront (`apps/storefront`)
- **Config**: `apps/storefront/tailwind.config.js`
- **Input CSS**: `apps/storefront/assets/input.css`
- **Output CSS**: `apps/storefront/static/output.css` (generated, ignored by git)

## Development Workflow

### Option 1: Using npm scripts (Recommended for CSS-only development)

```bash
# Watch mode for admin (rebuilds CSS on file changes)
npm run css:admin

# Watch mode for storefront
npm run css:storefront

# Production build for admin
npm run css:admin:build

# Production build for storefront
npm run css:storefront:build
```

### Option 2: Using Trunk (Integrated with Leptos CSR build)

```bash
# For admin panel (Trunk automatically runs Tailwind via hooks)
cd apps/admin
trunk serve

# For production build
trunk build --release
```

### Option 3: Using cargo-leptos (For SSR storefront)

```bash
cd apps/storefront
cargo leptos watch
```

## How It Works

### Admin Panel (CSR with Trunk)

1. **Trunk hooks** (defined in `apps/admin/Trunk.toml`) automatically run Tailwind CLI before building
2. During `trunk serve` (watch mode), Tailwind compiles in watch mode
3. During `trunk build` (production), Tailwind compiles with `--minify`

### Storefront (SSR with cargo-leptos)

1. Similar setup with cargo-leptos configuration
2. Tailwind CLI runs before Leptos compilation

## Adding Tailwind Classes

Just use Tailwind utility classes in your Leptos components:

```rust
use leptos::*;

#[component]
pub fn MyComponent() -> impl IntoView {
    view! {
        <div class="bg-blue-500 text-white p-4 rounded-lg hover:bg-blue-600">
            "Hello from Tailwind!"
        </div>
    }
}
```

Tailwind will automatically:
1. Scan your Rust files (`./src/**/*.rs`)
2. Extract class names
3. Generate only the CSS you use (tree-shaking)

## Customizing Tailwind

Edit `tailwind.config.js` in each app to customize:

```javascript
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
  ],
  theme: {
    extend: {
      colors: {
        brand: {
          500: '#your-color',
        },
      },
    },
  },
  plugins: [],
}
```

## Troubleshooting

### CSS not updating?

```bash
# Manually rebuild CSS
npm run css:admin:build
# or
npm run css:storefront:build

# Clear Trunk cache
rm -rf apps/admin/dist
trunk serve
```

### Classes not being generated?

1. Check that your Rust files are included in `content` array in `tailwind.config.js`
2. Ensure Tailwind CLI is watching the right files
3. Restart the dev server

### Node.js not found?

Ensure Node.js is installed:
```bash
node --version  # Should be v18+
npm --version
```

## CI/CD Integration

### Option 1: Generate CSS during build

```yaml
# In GitHub Actions or similar
- name: Install Node.js
  uses: actions/setup-node@v3
  with:
    node-version: '20'

- name: Install dependencies
  run: npm install

- name: Build Tailwind CSS
  run: |
    npm run css:admin:build
    npm run css:storefront:build

- name: Build Rust
  run: cargo build --release
```

### Option 2: Commit generated CSS (for zero-dependency builds)

If you want to avoid Node.js in CI:
1. Locally run `npm run css:admin:build && npm run css:storefront:build`
2. Remove `apps/*/output.css` from `.gitignore`
3. Commit the generated CSS files
4. CI just builds Rust, using pre-compiled CSS

## Why Not tailwind-rs?

The `tailwind-rs` Rust crate had blocking issues:
- ❌ Depends on outdated `parcel_css` v1.0.0-alpha.32
- ❌ Compilation errors with current Rust toolchain
- ❌ Not actively maintained
- ❌ Missing latest Tailwind features

Official Tailwind CLI:
- ✅ Actively maintained by Tailwind Labs
- ✅ All latest features (Tailwind v4 ready)
- ✅ Faster and more reliable
- ✅ Used by production Leptos projects
- ✅ Zero compilation issues

## Resources

- [Tailwind CSS Docs](https://tailwindcss.com/docs)
- [Leptos Tailwind Examples](https://github.com/leptos-rs/leptos/tree/main/examples/tailwind_actix)
- [Trunk Documentation](https://trunkrs.dev/)
- [cargo-leptos Documentation](https://github.com/leptos-rs/cargo-leptos)

## Related Files

- `package.json` - npm scripts and Tailwind dependency
- `apps/admin/Trunk.toml` - Trunk hooks for admin
- `apps/admin/tailwind.config.js` - Admin Tailwind config
- `apps/storefront/tailwind.config.js` - Storefront Tailwind config
- `.gitignore` - Excludes generated CSS files
