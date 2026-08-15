// Executes the real browser artifact against the golden corpus [playground.wasm].
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repository = path.resolve(__dirname, "..");
const { generate } = require(path.join(repository, "src/dmx/target/wasm-node/dmx_wasm.js"));
const input = fs.readFileSync(path.join(repository, "src/dmx/tests/golden/plain.dart"), "utf8");

const [generated, output] = generate(input);
assert.equal(generated, true);
assert.match(output, /static Result<Plain, DecodeError> fromJson/);
assert.match(output, /Plain copyWith/);
assert.match(output, /\/\/#region/);
assert.match(output, /\/\/#endregion/);

const [unchanged, secondOutput] = generate(output);
assert.equal(unchanged, true);
assert.equal(secondOutput, output);

const [valid, diagnostic] = generate("@Model() class Broken { final int ===; }");
assert.equal(valid, false);
assert.match(diagnostic, /DMX4001/);

console.log("WASM smoke test passed: generation, idempotence, and diagnostics");
