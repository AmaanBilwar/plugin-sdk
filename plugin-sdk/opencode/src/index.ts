import type { Hooks, Plugin, PluginInput, PluginOptions } from "@opencode-ai/plugin";

export * from "@opencode-ai/plugin";

export type HookPatch = Partial<Hooks>;
export type HookFactory = (
  ctx: PluginInput,
  options?: PluginOptions,
) => HookPatch | Promise<HookPatch>;

type BeforeToolExecuteHook = NonNullable<Hooks["tool.execute.before"]>;
type AfterToolExecuteHook = NonNullable<Hooks["tool.execute.after"]>;
type ShellEnvHook = NonNullable<Hooks["shell.env"]>;
type SessionIdleHandler = (event: unknown, ctx: PluginInput) => void | Promise<void>;
type EventHandler = (event: unknown, ctx: PluginInput) => void | Promise<void>;
type ReadToolBeforeHandler = (
  filePath: string,
  input: Parameters<BeforeToolExecuteHook>[0],
  output: Parameters<BeforeToolExecuteHook>[1],
  ctx: PluginInput,
) => void | Promise<void>;
type InjectEnvInput = Record<string, string | undefined> | ((
  input: Parameters<ShellEnvHook>[0],
  ctx: PluginInput,
) => Record<string, string | undefined> | Promise<Record<string, string | undefined>>);

export function definePlugin(factory: Plugin): Plugin {
  return factory;
}

export function withHooks(hooks: HookPatch): HookFactory {
  return async () => hooks;
}

export function onToolExecuteBefore(
  handler: (input: Parameters<BeforeToolExecuteHook>[0], output: Parameters<BeforeToolExecuteHook>[1], ctx: PluginInput) => void | Promise<void>,
): HookFactory {
  return async (ctx) => ({
    "tool.execute.before": async (input, output) => {
      await handler(input, output, ctx);
    },
  });
}

export function onToolExecuteAfter(
  handler: (input: Parameters<AfterToolExecuteHook>[0], output: Parameters<AfterToolExecuteHook>[1], ctx: PluginInput) => void | Promise<void>,
): HookFactory {
  return async (ctx) => ({
    "tool.execute.after": async (input, output) => {
      await handler(input, output, ctx);
    },
  });
}

export function onShellEnv(
  handler: (input: Parameters<ShellEnvHook>[0], output: Parameters<ShellEnvHook>[1], ctx: PluginInput) => void | Promise<void>,
): HookFactory {
  return async (ctx) => ({
    "shell.env": async (input, output) => {
      await handler(input, output, ctx);
    },
  });
}

export function injectEnv(envOrFactory: InjectEnvInput): HookFactory {
  return onShellEnv(async (input, output, ctx) => {
    const env =
      typeof envOrFactory === "function"
        ? await envOrFactory(input, ctx)
        : envOrFactory;

    for (const [key, value] of Object.entries(env)) {
      if (typeof value === "string") {
        output.env[key] = value;
      }
    }
  });
}

export function onEvent(eventType: string, handler: EventHandler): HookFactory {
  return async (ctx) => ({
    event: async ({ event }) => {
      const type = (event as { type?: unknown }).type;
      if (type === eventType) {
        await handler(event, ctx);
      }
    },
  });
}

export function onSessionIdle(handler: SessionIdleHandler): HookFactory {
  return onEvent("session.idle", handler);
}

export function onReadToolExecuteBefore(handler: ReadToolBeforeHandler): HookFactory {
  return onToolExecuteBefore(async (input, output, ctx) => {
    if (input.tool !== "read") {
      return;
    }
    const filePath = String((output.args as { filePath?: string }).filePath ?? "");
    await handler(filePath, input, output, ctx);
  });
}

export function blockReadPaths(patterns: Array<string | RegExp>, message = "Blocked by plugin policy"): HookFactory {
  return onReadToolExecuteBefore((filePath) => {
    const shouldBlock = patterns.some((pattern) =>
      typeof pattern === "string"
        ? filePath.includes(pattern)
        : pattern.test(filePath),
    );
    if (shouldBlock) {
      throw new Error(message);
    }
  });
}

export function addTools(tools: NonNullable<Hooks["tool"]>): HookFactory {
  return withHooks({ tool: tools });
}

export function createPlugin(...parts: HookFactory[]): Plugin {
  return definePlugin(async (ctx, options) => {
    const patches = await Promise.all(parts.map((part) => part(ctx, options)));
    return mergeHooks(patches);
  });
}

function mergeHooks(patches: HookPatch[]): Hooks {
  const merged: Hooks = {};
  const mergedTools: NonNullable<Hooks["tool"]> = {};

  for (const patch of patches) {
    for (const [key, value] of Object.entries(patch) as Array<[keyof Hooks, Hooks[keyof Hooks]]>) {
      if (key === "tool") {
        const toolPatch = value as Hooks["tool"] | undefined;
        if (!toolPatch) continue;
        for (const [toolName, definition] of Object.entries(toolPatch)) {
          mergedTools[toolName] = definition;
        }
        continue;
      }

      if (typeof value !== "undefined") {
        (merged as Record<string, unknown>)[key] = value;
      }
    }
  }

  if (Object.keys(mergedTools).length > 0) {
    merged.tool = mergedTools;
  }

  return merged;
}
