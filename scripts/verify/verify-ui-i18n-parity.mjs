import fs from "node:fs";
import path from "node:path";

const workspaceRoot = process.cwd();
const scanRoots = ["apps", "crates", "packages"];
const bundleDirs = new Set(["locales", "messages"]);
const localePairs = [
  ["en.json", "ru.json"],
];

function flattenJson(value, prefix = "") {
  if (Array.isArray(value)) {
    return value.flatMap((item, index) =>
      flattenJson(item, `${prefix}[${index}]`)
    );
  }

  if (value && typeof value === "object") {
    return Object.entries(value).flatMap(([key, child]) => {
      const nextPrefix = prefix ? `${prefix}.${key}` : key;
      return flattenJson(child, nextPrefix);
    });
  }

  return [prefix];
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function walkDirectories(rootPath, onDirectory) {
  if (!fs.existsSync(rootPath)) {
    return;
  }

  const stack = [rootPath];
  while (stack.length > 0) {
    const current = stack.pop();
    const entries = fs.readdirSync(current, { withFileTypes: true });
    onDirectory(current, entries);

    for (const entry of entries) {
      if (!entry.isDirectory()) {
        continue;
      }
      if (entry.name === "node_modules" || entry.name === "target" || entry.name.startsWith(".")) {
        continue;
      }
      stack.push(path.join(current, entry.name));
    }
  }
}

function discoverBundlePairs() {
  const results = [];

  for (const relativeRoot of scanRoots) {
    const absoluteRoot = path.join(workspaceRoot, relativeRoot);
    walkDirectories(absoluteRoot, (directory, entries) => {
      const dirName = path.basename(directory);
      if (!bundleDirs.has(dirName)) {
        return;
      }

      const fileNames = new Set(entries.filter((entry) => entry.isFile()).map((entry) => entry.name));
      for (const [left, right] of localePairs) {
        if (fileNames.has(left) && fileNames.has(right)) {
          results.push({
            directory,
            left: path.join(directory, left),
            right: path.join(directory, right),
          });
        }
      }
    });
  }

  return results.sort((left, right) => left.directory.localeCompare(right.directory));
}

function compareBundlePair(pair) {
  const leftJson = readJson(pair.left);
  const rightJson = readJson(pair.right);
  const leftKeys = new Set(flattenJson(leftJson).filter(Boolean));
  const rightKeys = new Set(flattenJson(rightJson).filter(Boolean));

  const missingOnRight = [...leftKeys].filter((key) => !rightKeys.has(key)).sort();
  const missingOnLeft = [...rightKeys].filter((key) => !leftKeys.has(key)).sort();

  return {
    ...pair,
    missingOnRight,
    missingOnLeft,
  };
}

const pairs = discoverBundlePairs();

if (pairs.length === 0) {
  console.error("No UI locale/message bundle pairs found.");
  process.exit(1);
}

let hasMismatch = false;

for (const pair of pairs) {
  const result = compareBundlePair(pair);
  const relativeDirectory = path.relative(workspaceRoot, pair.directory);

  if (result.missingOnRight.length === 0 && result.missingOnLeft.length === 0) {
    console.log(`OK  ${relativeDirectory}`);
    continue;
  }

  hasMismatch = true;
  console.error(`FAIL ${relativeDirectory}`);
  if (result.missingOnRight.length > 0) {
    console.error(`  Missing in ${path.basename(pair.right)}: ${result.missingOnRight.join(", ")}`);
  }
  if (result.missingOnLeft.length > 0) {
    console.error(`  Missing in ${path.basename(pair.left)}: ${result.missingOnLeft.join(", ")}`);
  }
}

if (hasMismatch) {
  process.exit(1);
}
