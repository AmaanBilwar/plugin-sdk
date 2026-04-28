use std::path::PathBuf;

use crate::generator::{GenerateOptions, Target, Template};

#[derive(Debug, Clone)]
pub struct RenderedTemplateFile {
    pub relative_path: PathBuf,
    pub contents: String,
}

pub fn render(options: &GenerateOptions) -> color_eyre::Result<Vec<RenderedTemplateFile>> {
    match (options.target, options.template) {
        (Target::OpenCode, Template::Minimal) => Ok(render_opencode_minimal(options)),
        (Target::Pi, Template::Minimal) => Ok(render_pi_minimal(options)),
    }
}

fn render_opencode_minimal(options: &GenerateOptions) -> Vec<RenderedTemplateFile> {
    let plugin_symbol = pascal_case_identifier(&options.project_name);
    let package_name = kebab_case_identifier(&options.project_name);
    let plugin_file_name = format!("{package_name}.ts");

    vec![
        RenderedTemplateFile {
            relative_path: PathBuf::from("package.json"),
            contents: format!(
                r#"{{
  "name": "{package_name}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "test": "echo \"No tests yet\""
  }},
  "dependencies": {{
    "@amaan/plugin-sdk": "{sdk_dependency}"
  }}
}}
"#,
                sdk_dependency = options.sdk_dependency
            ),
        },
        RenderedTemplateFile {
            relative_path: PathBuf::from(".opencode/package.json"),
            contents: format!(
                r#"{{
  "dependencies": {{
    "@amaan/plugin-sdk": "{sdk_dependency}",
    "@opencode-ai/plugin": "1.14.28"
  }}
}}
"#,
                sdk_dependency = options.sdk_dependency
            ),
        },
        RenderedTemplateFile {
            relative_path: PathBuf::from(format!(".opencode/plugins/{plugin_file_name}")),
            contents: format!(
                r#"import type {{ Plugin }} from "@amaan/plugin-sdk";
import {{ blockReadPaths, createPlugin, injectEnv, onSessionIdle }} from "@amaan/plugin-sdk";

export const {plugin_symbol}: Plugin = createPlugin(
  injectEnv((input) => ({{
    AMAAN_PLUGIN: "true",
    AMAAN_WORKTREE: input.cwd,
  }})),
  blockReadPaths([/\.md$/], "Blocked by {plugin_symbol}"),
  onSessionIdle(async (_event, ctx) => {{
    await ctx.client.app.log({{
      body: {{
        service: "{package_name}",
        level: "info",
        message: "Session became idle",
        extra: {{
          project: ctx.project,
          directory: ctx.directory,
        }},
      }},
    }});
  }}),
);

export default {plugin_symbol};
"#
            ),
        },
    ]
}

fn render_pi_minimal(options: &GenerateOptions) -> Vec<RenderedTemplateFile> {
    let extension_symbol = pascal_case_identifier(&options.project_name);
    let package_name = kebab_case_identifier(&options.project_name);
    let extension_file_name = format!("{package_name}.ts");

    vec![
        RenderedTemplateFile {
            relative_path: PathBuf::from("package.json"),
            contents: format!(
                r#"{{
  "name": "{package_name}",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "test": "echo \"No tests yet\""
  }},
  "dependencies": {{
    "@amaan/plugin-sdk": "{sdk_dependency}"
  }}
}}
"#,
                sdk_dependency = options.sdk_dependency
            ),
        },
        RenderedTemplateFile {
            relative_path: PathBuf::from(format!(".pi/extensions/{extension_file_name}")),
            contents: format!(
                r#"import type {{ PiExtensionFactory }} from "@amaan/plugin-sdk/pi";
import {{
  blockReadPaths,
  createExtension,
  defineCommand,
  onSessionStart,
  registerCommand,
  sendMessageOn,
}} from "@amaan/plugin-sdk/pi";

export const {extension_symbol}: PiExtensionFactory = createExtension(
  blockReadPaths([/\.md$/], "Reading markdown files is blocked by {extension_symbol}"),
  onSessionStart((_event, ctx) => {{
    ctx.ui.notify?.("{extension_symbol} loaded", "info");
  }}),
  sendMessageOn("session_start", async () => ({{
    content: "{extension_symbol} boot message",
    display: true,
  }})),
  registerCommand(
    "sdk-ping",
    defineCommand({{
      description: "Send a typed sdk ping message",
      handler: async (args: string, ctx) => {{
        const suffix = args.trim() ? `: ${{args.trim()}}` : "";
        ctx.ui.notify?.(`sdk-ping${{suffix}}`, "info");
      }},
    }}),
  ),
);

export default {extension_symbol};
"#
            ),
        },
    ]
}

fn kebab_case_identifier(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_dash = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn pascal_case_identifier(raw: &str) -> String {
    let mut out = String::new();
    let mut make_upper = true;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            if make_upper {
                out.push(ch.to_ascii_uppercase());
                make_upper = false;
            } else {
                out.push(ch);
            }
        } else {
            make_upper = true;
        }
    }
    if out.is_empty() {
        "GeneratedPlugin".to_string()
    } else {
        out
    }
}
