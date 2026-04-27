import type { Plugin } from "@opencode-ai/plugin";
import { blockReadPaths, createPlugin } from "../../src/index.js";

// Guard a specific tool call before OpenCode executes it.
export const ToolGuardPlugin: Plugin = createPlugin(
  blockReadPaths([/\.env$/], "Reading .env is blocked by ToolGuardPlugin"),
);
