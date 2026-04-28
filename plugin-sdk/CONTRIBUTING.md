# Contributing to @amaan/plugin-sdk

## Local setup

```bash
npm install
```

## Build

```bash
npm run clean
npm run build
```

## Project structure

- `src/core/`: shared internal abstraction utilities
- `src/index.ts`: top-level thin exports
- `opencode/src/`: OpenCode adapter
- `opencode/examples/`: OpenCode examples
- `pi/src/`: PI adapter
- `pi/examples/`: PI examples

## Using local SDK in another project

From a consumer project `package.json`:

```json
{
  "dependencies": {
    "@amaan/plugin-sdk": "file:../plugin-sdk"
  }
}
```

Then run:

```bash
npm install
```

## Adding a new platform adapter

1. Add adapter source under `<platform>/src/`.
2. Add minimal examples under `<platform>/examples/`.
3. Reuse `src/core` primitives where possible.
4. Add package export subpath in `package.json`.
5. Update `tsconfig.json` include paths.
6. Build and smoke-test imports.

## Safety guidelines

- Do not add helpers that imply runtime capabilities not present in the target platform.
- Keep adapter behavior aligned to official platform docs.
- Prefer explicit helper names over overloaded behavior.
