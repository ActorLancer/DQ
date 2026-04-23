import { mkdir, rm } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const __dirname = dirname(fileURLToPath(import.meta.url));
const packageRoot = resolve(__dirname, "..");
const openapiRoot = resolve(packageRoot, "..", "openapi");
const generatedRoot = resolve(packageRoot, "src", "generated");

const specs = [
  "audit",
  "billing",
  "catalog",
  "delivery",
  "iam",
  "ops",
  "recommendation",
  "search",
  "trade",
];

await rm(generatedRoot, { recursive: true, force: true });
await mkdir(generatedRoot, { recursive: true });

for (const spec of specs) {
  const input = resolve(openapiRoot, `${spec}.yaml`);
  const output = resolve(generatedRoot, `${spec}.ts`);
  await execFileAsync(
    "pnpm",
    ["exec", "openapi-typescript", input, "--output", output],
    { cwd: packageRoot },
  );
}
