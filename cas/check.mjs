import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const bundle = process.argv[2] ?? join(here, "..", "web", "public", "cas.js");

new Function(readFileSync(bundle, "utf8")).call(globalThis);

const cas = globalThis.kosmosCas;
if (!cas || cas.version !== "1") {
  console.error(`${bundle} did not install kosmosCas v1`);
  process.exit(1);
}

const table = readFileSync(join(here, "expected.txt"), "utf8")
  .split("\n")
  .map((line) => line.trim())
  .filter((line) => line && !line.startsWith("#"));

let failures = 0;

for (const line of table) {
  const [method, input, wanted] = line.split(" | ").map((part) => part.trim());
  let got;
  if (method === "taylor") {
    const [source, about, order] = input.split(" @ ");
    got = cas.taylor(source, "x", Number(about), Number(order));
  } else {
    got = cas[method](input, "x");
  }
  const text = got.ok ? got.text : `!${got.text}`;
  if (text !== wanted) {
    failures += 1;
    console.error(`${method}(${input})\n  got    ${text}\n  wanted ${wanted}`);
  }
}

console.log(`${table.length} cases, ${failures} failures`);
process.exit(failures === 0 ? 0 : 1);
