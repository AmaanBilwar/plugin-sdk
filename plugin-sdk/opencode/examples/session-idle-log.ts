import type { Plugin } from "@opencode-ai/plugin";
import { createPlugin, onSessionIdle } from "../../src/index.js";

// Listen for session idle and emit a structured log line.
export const SessionIdleLogPlugin: Plugin = createPlugin(
  onSessionIdle(async (_event, ctx) => {
    await ctx.client.app.log({
      body: {
        service: "session-idle-log-plugin",
        level: "info",
        message: "Session idle",
        extra: {
          directory: ctx.directory,
        },
      },
    });
  }),
);
