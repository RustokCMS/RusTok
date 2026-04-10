import fs from "node:fs";
import path from "node:path";

const workspaceRoot = process.cwd();
const failures = [];

function readWorkspaceFile(relativePath) {
  return fs.readFileSync(path.join(workspaceRoot, relativePath), "utf8");
}

function expectContains(relativePath, expectedSnippet, description) {
  const content = readWorkspaceFile(relativePath);
  if (!content.includes(expectedSnippet)) {
    failures.push(`${relativePath}: expected ${description}`);
  }
}

function expectNotContains(relativePath, unexpectedSnippet, description) {
  const content = readWorkspaceFile(relativePath);
  if (content.includes(unexpectedSnippet)) {
    failures.push(`${relativePath}: found ${description}`);
  }
}

expectContains(
  "apps/server/migration/src/lib.rs",
  "m20260410_000001_cleanup_flex_attached_legacy_inline_metadata",
  "flex attached cleanup migration to be wired into the server migrator",
);
expectContains(
  "apps/server/src/services/flex_standalone_service.rs",
  "let resolved_localized = localized_data.and_then(|value| value.as_object().cloned());",
  "standalone entry view to resolve only parallel localized rows",
);
expectNotContains(
  "apps/server/src/services/flex_standalone_service.rs",
  "or_else(|| {\n                if legacy_localized.is_empty() {",
  "legacy inline localized fallback branch in standalone service",
);
expectContains(
  "crates/flex/src/attached.rs",
  "or_else(|| localized_by_locale.values().next().cloned())",
  "attached payload resolution to fall back only to existing localized rows",
);
expectNotContains(
  "crates/flex/src/attached.rs",
  "unwrap_or_else(|| Value::Object(legacy_localized.into_iter().collect()))",
  "legacy inline localized fallback in attached update path",
);
expectNotContains(
  "crates/flex/src/attached.rs",
  "Some(legacy_localized)",
  "legacy inline localized fallback in attached read path",
);
expectContains(
  "crates/flex/README.md",
  "Cleanup migrations remove residual inline locale-aware Flex payloads",
  "flex README to document migration-based cleanup",
);
expectContains(
  "crates/flex/docs/README.md",
  "runtime path не должен читать donor/base-row inline localized JSON как канонический fallback",
  "flex docs to ban inline localized runtime fallback",
);

if (failures.length > 0) {
  console.error("flex multilingual contract drift detected:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("OK  flex multilingual contract");
