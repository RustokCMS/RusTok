use super::*;

pub(crate) fn extract_runtime_module_dependencies(
    module_root: &Path,
) -> Result<Option<HashSet<String>>> {
    let lib_path = module_root.join("src").join("lib.rs");
    if !lib_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&lib_path)
        .with_context(|| format!("Failed to read {}", lib_path.display()))?;
    if !content.contains("impl RusToKModule for") {
        return Ok(None);
    }

    let marker = "fn dependencies(&self)";
    let Some(marker_index) = content.find(marker) else {
        return Ok(Some(HashSet::new()));
    };

    let tail = &content[marker_index..];
    let Some(body_start_offset) = tail.find('{') else {
        return Ok(Some(HashSet::new()));
    };
    let body = &tail[body_start_offset..];
    let Some(array_start_offset) = body.find("&[") else {
        return Ok(Some(HashSet::new()));
    };
    let array_tail = &body[(array_start_offset + 2)..];
    let Some(array_end_offset) = array_tail.find(']') else {
        anyhow::bail!(
            "Failed to parse RusToKModule::dependencies() in {}",
            lib_path.display()
        );
    };
    let array_body = &array_tail[..array_end_offset];

    Ok(Some(
        extract_string_literals(array_body)
            .into_iter()
            .collect::<HashSet<_>>(),
    ))
}

pub(crate) fn infer_runtime_module_entry_type(module_root: &Path) -> Result<Option<String>> {
    let lib_path = module_root.join("src").join("lib.rs");
    if !lib_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&lib_path)
        .with_context(|| format!("Failed to read {}", lib_path.display()))?;
    Ok(extract_runtime_module_entry_type(&content))
}

pub(crate) fn extract_runtime_module_entry_type(content: &str) -> Option<String> {
    let marker = "impl RusToKModule for ";
    let start = content.find(marker)? + marker.len();
    let ident = content[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>();
    if ident.is_empty() {
        return None;
    }

    Some(ident)
}

pub(crate) fn extract_runtime_module_kind(content: &str) -> Option<&'static str> {
    let body = extract_runtime_method_body(content, "kind")?;
    if body.contains("ModuleKind::Core") {
        return Some("Core");
    }
    if body.contains("ModuleKind::Optional") {
        return Some("Optional");
    }
    None
}

pub(crate) fn extract_runtime_method_body<'a>(
    content: &'a str,
    method_name: &str,
) -> Option<&'a str> {
    let marker = format!("fn {method_name}(&self)");
    let marker_index = content.find(&marker)?;
    let tail = &content[marker_index..];
    let body_start_offset = tail.find('{')?;
    let body = &tail[(body_start_offset + 1)..];
    let mut depth = 1usize;

    for (index, ch) in body.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&body[..index]);
                }
            }
            _ => {}
        }
    }

    None
}

pub(crate) fn extract_string_literals(input: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escape = false;

    for ch in input.chars() {
        if !in_string {
            if ch == '"' {
                in_string = true;
                current.clear();
            }
            continue;
        }

        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        match ch {
            '\\' => escape = true,
            '"' => {
                values.push(current.clone());
                current.clear();
                in_string = false;
            }
            _ => current.push(ch),
        }
    }

    values
}

pub(crate) fn extract_runtime_string_method(content: &str, method_name: &str) -> Option<String> {
    let marker = format!("fn {method_name}(&self)");
    let marker_index = content.find(&marker)?;
    let tail = &content[marker_index..];
    let body_start_offset = tail.find('{')?;
    let body = &tail[body_start_offset..];
    let first_quote_offset = body.find('"')?;
    let quoted = &body[(first_quote_offset + 1)..];
    let end_quote_offset = quoted.find('"')?;
    Some(quoted[..end_quote_offset].to_string())
}

pub(crate) fn collect_module_source_files(module_root: &Path) -> Result<Vec<(PathBuf, String)>> {
    let src_root = module_root.join("src");
    if !src_root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    collect_rust_source_files_recursive(&src_root, &mut files)?;
    Ok(files)
}

fn collect_rust_source_files_recursive(
    dir: &Path,
    files: &mut Vec<(PathBuf, String)>,
) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry =
            entry.with_context(|| format!("Failed to read entry under {}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files_recursive(&path, files)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        files.push((path, content));
    }

    Ok(())
}

pub(crate) fn provided_path_symbol(path: &str) -> Result<&str> {
    let symbol = path
        .trim()
        .rsplit("::")
        .next()
        .filter(|value| !value.trim().is_empty())
        .with_context(|| format!("Failed to resolve symbol from declared path '{path}'"))?;
    Ok(symbol)
}

pub(crate) fn validate_declared_symbol_exists(
    slug: &str,
    field: &str,
    declared_path: &str,
    symbol: &str,
    source_files: &[(PathBuf, String)],
) -> Result<()> {
    let found = source_files.iter().any(|(_, content)| {
        content.contains(&format!("pub struct {symbol}"))
            || content.contains(&format!("struct {symbol}"))
            || content.contains(&format!("pub enum {symbol}"))
            || content.contains(&format!("enum {symbol}"))
            || content.contains(&format!("pub fn {symbol}("))
            || content.contains(&format!("fn {symbol}("))
            || content.contains(symbol)
    });

    if !found {
        anyhow::bail!(
            "Module '{slug}' declares {field}='{}', but symbol '{}' was not found under src/**/*.rs",
            declared_path,
            symbol
        );
    }

    Ok(())
}
