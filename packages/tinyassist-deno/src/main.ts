import { run } from "@stricli/core";
import { config } from "dotenv";
import process from "node:process";
import { app } from "./app.ts";

if (import.meta.main) {
  config({ path: ".env" });
  await run(app, Deno.args, { process });
}
