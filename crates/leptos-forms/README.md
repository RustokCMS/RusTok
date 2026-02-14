# leptos-forms

Form handling и validation для Leptos приложений.

## Features

- **Form state management** — поля, значения, изменения
- **Validation rules** — required, email, min_length, custom
- **Error display** — per-field, form-level
- **Submit handling** — loading, error states
- **Reactive validation** — on blur, on change, on submit

## Installation

```toml
[dependencies]
leptos-forms = { path = "../../crates/leptos-forms" }
```

## Usage

### Basic Form

```rust
use leptos::*;
use leptos_forms::{use_form, Field, Validator};

#[derive(Default, Clone)]
struct LoginData {
    email: String,
    password: String,
}

#[component]
fn LoginForm() -> impl IntoView {
    let form = use_form(|| LoginData::default())
        .field("email", Validator::email().required())
        .field("password", Validator::min_length(6).required())
        .on_submit(|data| async move {
            api::sign_in(&data.email, &data.password).await
        });

    view! {
        <form on:submit=form.submit>
            <Field 
                form=form 
                name="email" 
                label="Email" 
                placeholder="you@example.com"
            />
            <Field 
                form=form 
                name="password" 
                label="Password" 
                type="password"
            />
            
            <button disabled=form.is_submitting>
                {move || if form.is_submitting() { "Loading..." } else { "Login" }}
            </button>
            
            {move || form.error().map(|err| view! {
                <div class="text-red-500">{err}</div>
            })}
        </form>
    }
}
```

### Validators

```rust
use leptos_forms::Validator;

// Required field
Validator::required()

// Email validation
Validator::email().required()

// Min/max length
Validator::min_length(6)
Validator::max_length(255)

// Pattern (regex)
Validator::pattern(r"^\d{3}-\d{3}-\d{4}$")

// Custom validator
Validator::custom(|value| {
    if value.contains("@") {
        Ok(())
    } else {
        Err("Must contain @".to_string())
    }
})
```

### Form API

```rust
// use_form hook returns FormContext
let form = use_form(|| MyData::default());

// Add field validators
form.field("email", Validator::email().required());

// Set submit handler
form.on_submit(|data| async move {
    // ... submit logic
});

// Check form state
form.is_submitting() -> bool
form.is_valid() -> bool
form.error() -> Option<String>

// Get field errors
form.get_field_error("email") -> Option<String>

// Set form-level error
form.set_error("Invalid credentials");

// Reset form
form.reset();
```

## Compatibility

- ✅ CSR (Client-Side Rendering)
- ✅ SSR (Server-Side Rendering)
- Leptos: 0.6+

## License

MIT OR Apache-2.0
