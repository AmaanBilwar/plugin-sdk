import type { Plugin } from "@opencode-ai/plugin";
import {
  blockReadPaths,
  createPlugin,
  injectEnv,
  onSessionIdle,
} from "../../src/index.js";

export const MinimalPlugin: Plugin = createPlugin(
  // 1) Inject env vars for every shell session.
  injectEnv((input) => ({
    AMAAN_PLUGIN: "true",
    AMAAN_WORKTREE: input.cwd,
  })),

  // 2) Block attempts to read env files.
  blockReadPaths([".env"], "Refusing to read .env files"),

  // 3) Log when a session becomes idle.
  onSessionIdle(async (_event, ctx) => {
    await ctx.client.app.log({
      body: {
        service: "minimal-plugin",
        level: "info",
        message: "Session became idle",
        extra: {
          project: ctx.project,
          directory: ctx.directory,
        },
      },
    });
  }),
);
