import type { Plugin } from "@opencode-ai/plugin";
import { createPlugin, injectEnv } from "../../src/index.js";

// Smallest possible example: inject environment variables.
export const EnvInjectionPlugin: Plugin = createPlugin(
  injectEnv((input) => ({
    MY_PLUGIN_ENABLED: "1",
    PROJECT_ROOT: input.cwd,
  })),
);
