# @soarailabs/plugin-sdk

Wrapper SDK for building plugins with less boilerplate.

> **Note:** This SDK currently supports only OpenCode plugins.
> Support for Codex plugins and pi coding agent extensions is planned soon.

## Install

```bash
npm install @soarailabs/plugin-sdk
```

## Write a plugin

Create a file in your project:

- `.opencode/plugins/my-plugin.ts`

Use the SDK helpers:

```ts
import type { Plugin } from "@soarailabs/plugin-sdk";
import { createPlugin, injectEnv, blockReadPaths, onSessionIdle } from "@soarailabs/plugin-sdk";

export const MyPlugin: Plugin = createPlugin(
  injectEnv(() => ({ MY_PLUGIN: "1" })),
  blockReadPaths([".env"], "Do not read .env files"),
  onSessionIdle(async (_event, ctx) => {
    await ctx.client.app.log({
      body: {
        service: "my-plugin",
        level: "info",
        message: "Session idle",
      },
    });
  }),
);

export default MyPlugin;
```

Restart OpenCode after adding or changing plugin files.

## Build this SDK

```bash
npm install
npm run build
```

## Available helpers

- `createPlugin`
- `withHooks`
- `onEvent`
- `onToolExecuteBefore`
- `onToolExecuteAfter`
- `onReadToolExecuteBefore`
- `blockReadPaths`
- `onShellEnv`
- `injectEnv`
- `onSessionIdle`
- `addTools`

## Examples

OpenCode examples are in `opencode/examples/`.
