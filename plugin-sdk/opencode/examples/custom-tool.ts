import type { Plugin } from "@opencode-ai/plugin";
import { addTools, createPlugin, tool } from "../../src/index.js";

// Add one custom tool that OpenCode can call.
export const CustomToolPlugin: Plugin = createPlugin(
  addTools({
    hello: tool({
      description: "Return a hello message from the plugin.",
      args: {
        name: tool.schema.string().default("friend"),
      },
      async execute(args, context) {
        return `Hello ${args.name} from ${context.directory}`;
      },
    }),
  }),
);
