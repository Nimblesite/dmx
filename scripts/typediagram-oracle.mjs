// Regenerates the typeDiagram compatibility corpus from the upstream package
// [typediagram.delivery.baseline].
//
// This is a DEVELOPMENT tool. The dmx binary never runs it, never depends on
// Node, and never calls typeDiagram: production parsing and model construction
// are the Rust front end's, and Mustache is the only authority on code shape.
// What this script produces is the *oracle* — the model JSON upstream's own
// parser and model builder emit for each fixture — which the Rust differential
// test compares against so that upstream language drift is visible.
//
// Usage:
//
//   node scripts/typediagram-oracle.mjs --typediagram <path-to-typeDiagram-checkout>
//
// The checkout must have been built (`npm run build` in the typeDiagram repo),
// because the script imports its compiled `dist`. Each `*.td` fixture in
// src/dmx/tests/typediagram/corpus is written back as `<name>.model.json`, and
// the exit status is non-zero if any fixture fails to parse or resolve.

import { readdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const CORPUS = resolve(HERE, "..", "src", "dmx", "tests", "typediagram", "corpus");

function upstreamDirectory() {
  const flag = process.argv.indexOf("--typediagram");
  const named = flag === -1 ? process.env.TYPEDIAGRAM_DIR : process.argv[flag + 1];
  if (named === undefined || named === "") {
    console.error("usage: node scripts/typediagram-oracle.mjs --typediagram <checkout>");
    console.error("   or: TYPEDIAGRAM_DIR=<checkout> node scripts/typediagram-oracle.mjs");
    process.exit(2);
  }
  return resolve(named);
}

async function load(upstream) {
  const dist = join(upstream, "packages", "typediagram", "dist");
  const importFrom = (relative) => import(pathToFileURL(join(dist, relative)).href);
  const [parser, model, json] = await Promise.all([
    importFrom("parser/index.js"),
    importFrom("model/index.js"),
    importFrom("model/json.js"),
  ]);
  return { parse: parser.parse, buildModel: model.buildModel, toJSON: json.toJSON };
}

function describe(diagnostics) {
  return diagnostics
    .map((d) => `${String(d.line)}:${String(d.col)} ${d.severity} ${d.message}`)
    .join("\n");
}

const upstream = upstreamDirectory();
const { parse, buildModel, toJSON } = await load(upstream);

const fixtures = (await readdir(CORPUS)).filter((name) => name.endsWith(".td")).sort();
let failed = 0;

for (const fixture of fixtures) {
  const source = await readFile(join(CORPUS, fixture), "utf8");
  const ast = parse(source);
  if (!ast.ok) {
    console.error(`${fixture}: parse failed\n${describe(ast.error)}`);
    failed += 1;
    continue;
  }
  const built = buildModel(ast.value);
  if (!built.ok) {
    console.error(`${fixture}: model failed\n${describe(built.error)}`);
    failed += 1;
    continue;
  }
  const target = join(CORPUS, `${fixture.slice(0, -3)}.model.json`);
  await writeFile(target, `${JSON.stringify(toJSON(built.value), null, 2)}\n`, "utf8");
  console.log(`wrote ${target}`);
}

if (failed > 0) {
  console.error(`${String(failed)} fixture(s) failed`);
  process.exit(1);
}
