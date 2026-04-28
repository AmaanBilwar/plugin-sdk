import {
  matchesAnyPattern,
  runRuntimeParts,
  type MaybePromise,
  type RuntimePart,
} from "../../src/core/index.js";

export type PiNotificationLevel = "info" | "success" | "warn" | "error";

export type PiUI = {
  notify?(message: string, level?: PiNotificationLevel): void;
  confirm?(title: string, message: string): Promise<boolean>;
  input?(title: string, prompt: string): Promise<string | undefined>;
  setStatus?(key: string, value: string): void;
  setWidget?(key: string, lines: string[]): void;
};

export type PiExtensionContext = {
  cwd: string;
  ui: PiUI;
  signal?: AbortSignal;
  [key: string]: unknown;
};

export type PiExtensionCommandContext = PiExtensionContext;

export type PiToolCallEvent = {
  toolName?: string;
  toolCallId?: string;
  input?: Record<string, unknown>;
};

export type PiSessionStartEvent = {
  reason?: "startup" | "reload" | "new" | "resume" | "fork";
  previousSessionFile?: string;
};

export type PiInputEvent = {
  text: string;
  source?: "interactive" | "rpc" | "extension";
  images?: unknown[];
};

export type PiUserBashEvent = {
  command: string;
  cwd: string;
  excludeFromContext?: boolean;
};

export type PiMessageRenderMessage = {
  customType?: string;
  content: string;
  details?: unknown;
};

export type PiMessageRendererOptions = {
  expanded?: boolean;
  [key: string]: unknown;
};

export type PiMessageRenderer = (
  message: PiMessageRenderMessage,
  options: PiMessageRendererOptions,
  theme: unknown,
) => unknown;
export type PiSendMessageInput = {
  customType?: string;
  content: string;
  details?: unknown;
  display?: boolean;
};

export type PiSendMessageOptions = {
  triggerTurn?: boolean;
  [key: string]: unknown;
};

export type PiEventMap = {
  tool_call: PiToolCallEvent;
  session_start: PiSessionStartEvent;
  input: PiInputEvent;
  user_bash: PiUserBashEvent;
};

export type PiExtensionAPI = {
  on<E extends keyof PiEventMap>(
    eventName: E,
    handler: (event: PiEventMap[E], ctx: PiExtensionContext) => MaybePromise<unknown>,
  ): void;
  on(
    eventName: string,
    handler: (event: unknown, ctx: PiExtensionContext) => MaybePromise<unknown>,
  ): void;
  registerTool(definition: PiToolDefinition): void;
  registerCommand<TArgs extends string = string>(
    name: string,
    options: PiCommandDefinition<TArgs>,
  ): void;
  registerMessageRenderer(customType: string, renderer: PiMessageRenderer): void;
  sendMessage(message: PiSendMessageInput, options?: PiSendMessageOptions): void;
};

export type PiToolDefinition = Record<string, unknown>;
export type PiCommandDefinition<TArgs extends string = string> = {
  description?: string;
  handler: (args: TArgs, ctx: PiExtensionCommandContext) => MaybePromise<void>;
} & Record<string, unknown>;
export type PiBlockResult = {
  block: true;
  reason?: string;
};

export type PiExtensionFactory = (pi: PiExtensionAPI) => void | Promise<void>;
export type PiExtensionPart = RuntimePart<PiExtensionAPI>;

type ToolCallHandler = (
  event: PiToolCallEvent,
  ctx: PiExtensionContext,
) => void | Promise<void> | PiBlockResult;

type ReadToolCallHandler = (
  path: string,
  event: PiToolCallEvent,
  ctx: PiExtensionContext,
) => void | Promise<void> | PiBlockResult;

export function defineExtension(factory: PiExtensionFactory): PiExtensionFactory {
  return factory;
}

export function withExtension(setup: PiExtensionPart): PiExtensionPart {
  return setup;
}

export function createExtension(...parts: PiExtensionPart[]): PiExtensionFactory {
  return defineExtension(async (pi) => {
    await runRuntimeParts(parts, pi);
  });
}

export function onEvent<TEvent = unknown>(
  eventName: string,
  handler: (event: TEvent, ctx: PiExtensionContext) => MaybePromise<unknown>,
): PiExtensionPart {
  return withExtension((pi) => {
    pi.on(eventName, async (event, ctx) => handler(event as TEvent, ctx));
  });
}

export function onToolCall(handler: ToolCallHandler): PiExtensionPart {
  return onEvent<PiToolCallEvent>("tool_call", (event, ctx) => handler(event, ctx));
}

export function onSessionStart(
  handler: (event: PiSessionStartEvent, ctx: PiExtensionContext) => MaybePromise<unknown>,
): PiExtensionPart {
  return onEvent<PiSessionStartEvent>("session_start", (event, ctx) => handler(event, ctx));
}

export function onReadToolCall(handler: ReadToolCallHandler): PiExtensionPart {
  return onToolCall((event, ctx) => {
    if (event.toolName !== "read") {
      return;
    }
    const path = String(
      event.input?.path ??
        event.input?.filePath ??
        "",
    );
    return handler(path, event, ctx);
  });
}

export function blockReadPaths(
  patterns: Array<string | RegExp>,
  reason = "Blocked by extension policy",
): PiExtensionPart {
  return onReadToolCall((path) => {
    if (matchesAnyPattern(path, patterns)) {
      return { block: true, reason };
    }
  });
}

export function addTools(
  tools: PiToolDefinition[] | Record<string, PiToolDefinition>,
): PiExtensionPart {
  return withExtension((pi) => {
    const list = Array.isArray(tools) ? tools : Object.values(tools);
    for (const tool of list) {
      pi.registerTool(tool);
    }
  });
}

export function defineCommand<TArgs extends string = string>(
  definition: PiCommandDefinition<TArgs>,
): PiCommandDefinition<TArgs> {
  return definition;
}

export function registerCommand<TArgs extends string = string>(
  name: string,
  options: PiCommandDefinition<TArgs>,
): PiExtensionPart {
  return withExtension((pi) => {
    pi.registerCommand(name, options);
  });
}

export function registerMessageRenderer(
  customType: string,
  renderer: PiMessageRenderer,
): PiExtensionPart {
  return withExtension((pi) => {
    pi.registerMessageRenderer(customType, renderer);
  });
}

export function sendMessage(
  message: PiSendMessageInput,
  options?: PiSendMessageOptions,
): PiExtensionPart {
  return withExtension((pi) => {
    pi.sendMessage(message, options);
  });
}

export function sendMessageOn<E extends keyof PiEventMap>(
  eventName: E,
  builder: (
    event: PiEventMap[E],
    ctx: PiExtensionContext,
  ) => MaybePromise<PiSendMessageInput | undefined | null>,
  options?: PiSendMessageOptions,
): PiExtensionPart {
  return withExtension((pi) => {
    pi.on(eventName, async (event, ctx) => {
      const message = await builder(event as PiEventMap[E], ctx);
      if (!message) return;
      pi.sendMessage(message, options);
    });
  });
}
