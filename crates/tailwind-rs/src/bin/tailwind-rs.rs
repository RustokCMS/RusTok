//! `tailwind-rs` CLI — scans Rust/Leptos source files for Tailwind class strings
//! and generates a CSS bundle using the vendored `tailwind-css` crate.
//!
//! Usage:
//!   tailwind-rs -i <src_dir>  -o <output.css>  [--append <file.css>]  [--minify]
//!
//! The scanner looks for every `class="..."` and `attr:class="..."` occurrence in
//! `.rs` files under `<src_dir>` and feeds the raw class strings to TailwindBuilder.
//! `bundle()` then emits preflight + all collected utility rules.
//!
//! Note: `tailwind-css` generates its own class-name scheme for some utilities
//! (e.g. `.display-flex` instead of `.flex`).  Pass `--append input.css` to
//! add bridge/override rules that map standard Tailwind names to the right CSS.

use std::{
    env,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use tailwind_css::TailwindBuilder;
use tailwind_rs::CLIConfig;
use walkdir::WalkDir;

fn main() {
    // ── parse args ──────────────────────────────────────────────────────────
    let args: Vec<String> = env::args().collect();
    let mut src_dir: Option<PathBuf> = None;
    let mut out_path: Option<PathBuf> = None;
    let mut append_path: Option<PathBuf> = None;
    let mut minify = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-i" => {
                i += 1;
                src_dir = Some(PathBuf::from(&args[i]));
            },
            "-o" => {
                i += 1;
                out_path = Some(PathBuf::from(&args[i]));
            },
            "--append" => {
                i += 1;
                append_path = Some(PathBuf::from(&args[i]));
            },
            "--minify" => minify = true,
            other => {
                eprintln!("tailwind-rs: unknown argument `{}`", other);
                std::process::exit(1);
            },
        }
        i += 1;
    }

    let src_dir = src_dir.unwrap_or_else(|| PathBuf::from("src"));
    let out_path = out_path.unwrap_or_else(|| PathBuf::from("dist/output.css"));

    // ── scan sources ────────────────────────────────────────────────────────
    let mut builder = TailwindBuilder::default();
    scan_dir(&src_dir, &mut builder);

    // ── generate CSS ─────────────────────────────────────────────────────────
    let mut css = match builder.bundle() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("tailwind-rs: bundle failed: {}", e);
            std::process::exit(1);
        },
    };

    // ── append custom CSS (bridge rules, overrides) ──────────────────────────
    if let Some(ref path) = append_path {
        match fs::read_to_string(path) {
            Ok(extra) => {
                css.push('\n');
                css.push_str(&extra);
            },
            Err(e) => {
                eprintln!("tailwind-rs: cannot read append file `{}`: {}", path.display(), e);
                std::process::exit(1);
            },
        }
    }

    // ── optional minify via parcel_css ──────────────────────────────────────
    let css = if minify {
        let config = CLIConfig { minify: true, ..Default::default() };
        match config.compile_css(&css) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("tailwind-rs: minify failed: {}", e);
                css
            },
        }
    } else {
        css
    };

    // ── write output ─────────────────────────────────────────────────────────
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).unwrap_or_default();
        }
    }
    match fs::File::create(&out_path).and_then(|mut f| f.write_all(css.as_bytes())) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("tailwind-rs: write `{}` failed: {}", out_path.display(), e);
            std::process::exit(1);
        },
    }
}

/// Recursively walk `dir`, visit every `.rs` file, and extract Tailwind
/// class strings into `builder`.
fn scan_dir(dir: &Path, builder: &mut TailwindBuilder) {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "rs") {
            if let Ok(source) = fs::read_to_string(path) {
                extract_classes(&source, builder);
            }
        }
    }
}

/// Extract every `class="..."` / `attr:class="..."` value from a Rust source
/// string and register it with `builder.trace()`.
///
/// We look for the literal bytes `class="` (and `class=` followed by a quoted
/// string that could be a Rust string literal). This handles the common Leptos
/// patterns:
///   class="flex items-center gap-2"
///   attr:class="..."
fn extract_classes(source: &str, builder: &mut TailwindBuilder) {
    let markers: &[&str] = &["class=\"", "attr:class=\""];

    for marker in markers {
        let mut rest = source;
        while let Some(start) = rest.find(marker) {
            rest = &rest[start + marker.len()..];
            if let Some(end) = rest.find('"') {
                let class_str = &rest[..end];
                if !class_str.is_empty() {
                    if let Err(e) = builder.trace(class_str, false) {
                        log::warn!("tailwind-rs: skipped `{}`: {}", class_str, e);
                    }
                }
                rest = &rest[end + 1..];
            } else {
                break;
            }
        }
    }
}
