# @amaan/plugin-sdk

while I work on docs and minimal examples on how to use this sdk, feel free to try it out yourself :) 

## What this project is

`@amaan/plugin-sdk` is a wrapper SDK for coding-agent extension systems.
It gives a shared abstraction layer over multiple agent runtimes.

## What it does

It reduces extension/plugin boilerplate by providing reusable helpers such as:

- lifecycle/event registration
- policy helpers (for example file-read guards)
- composable extension/plugin construction
- platform-specific adapters under one package

## Why it does it

Each coding agent has a different extension API surface.
Without a wrapper, teams rewrite the same patterns for every platform.
This project provides a consistent authoring model so extension logic is easier to write, test, and port.

## How it does it

The SDK is organized into:

- a shared internal core abstraction layer (`src/core`)
- platform adapters (`opencode`, `pi`)
- thin top-level exports (`src/index.ts`)

The shared core contains composition primitives.
Each platform adapter maps those primitives to its native runtime APIs.

## What plugins/extensions are supported

Current support:

- OpenCode plugins (`@amaan/plugin-sdk/opencode`)
- PI coding-agent extensions (`@amaan/plugin-sdk/pi`)

Note for PI:

- PI does not have an OpenCode-style global `shell.env` hook.
- The PI adapter intentionally avoids exposing behavior that PI does not natively support.

## Future coding-agent plugins/extensions support

Planned next support areas:

- Codex-focused wrappers
- additional coding-agent extension systems as they stabilize
- stronger typed event/context maps per platform

---

For installation, local development, build commands, and contribution workflow, see `CONTRIBUTING.md`.
