use std::env;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use crate::generator::{self, GenerateOptions, GenerateResult, Target, Template};

enum Command {
    Generate(GenerateArgs),
    Help,
}

#[derive(Debug, Clone)]
struct GenerateArgs {
    target: Option<Target>,
    template: Template,
    name: Option<String>,
    output: Option<PathBuf>,
    force: bool,
    dry_run: bool,
    install_deps: bool,
}

pub fn run() -> color_eyre::Result<()> {
    match parse_command(env::args().skip(1).collect())? {
        Command::Generate(args) => run_generate(args),
        Command::Help => {
            print_help();
            Ok(())
        }
    }
}

fn run_generate(args: GenerateArgs) -> color_eyre::Result<()> {
    let sdk_dependency = resolve_sdk_dependency()?;
    let install_deps;
    let options;

    if let (Some(target), Some(name)) = (args.target, args.name.clone()) {
        let output_dir = args.output.unwrap_or(
            env::current_dir()
                .map_err(|e| color_eyre::eyre::eyre!(e))?
                .join(&name),
        );
        install_deps = args.install_deps;
        options = GenerateOptions {
            target,
            template: args.template,
            project_name: name,
            output_dir,
            sdk_dependency: sdk_dependency.clone(),
            force: args.force,
            dry_run: args.dry_run,
        };
    } else {
        let mut terminal = ratatui::init();
        let wizard_output = crate::tui::run_generate_wizard(&mut terminal);
        ratatui::restore();
        let wizard_output = wizard_output?;
        install_deps = wizard_output.install_deps;
        options = GenerateOptions {
            sdk_dependency,
            ..wizard_output.options
        };
    }

    let result = generator::generate(&options)?;
    print_result(&result);

    if install_deps && !options.dry_run {
        install_dependencies(&options)?;
    }

    Ok(())
}

fn install_dependencies(options: &GenerateOptions) -> color_eyre::Result<()> {
    run_npm_install(&options.output_dir)?;
    if matches!(options.target, Target::OpenCode) {
        run_npm_install(&options.output_dir.join(".opencode"))?;
    }
    Ok(())
}

fn run_npm_install(dir: &Path) -> color_eyre::Result<()> {
    let package_json = dir.join("package.json");
    if !package_json.exists() {
        return Ok(());
    }

    println!("Installing dependencies in {}...", dir.display());
    let status = ProcessCommand::new("npm")
        .arg("install")
        .current_dir(dir)
        .status()
        .map_err(|e| color_eyre::eyre::eyre!(e))?;
    if !status.success() {
        return Err(color_eyre::eyre::eyre!(
            "npm install failed in {}",
            dir.display()
        ));
    }
    Ok(())
}

fn resolve_sdk_dependency() -> color_eyre::Result<String> {
    let cwd = env::current_dir().map_err(|e| color_eyre::eyre::eyre!(e))?;
    let candidates = [
        cwd.join("../plugin-sdk"),
        cwd.join("plugin-sdk"),
        cwd.join("../../plugin-sdk"),
    ];
    for candidate in candidates {
        if candidate.exists() {
            let absolute = candidate
                .canonicalize()
                .map_err(|e| color_eyre::eyre::eyre!(e))?;
            return Ok(format!("file:{}", absolute.display()));
        }
    }
    Ok("file:../plugin-sdk".to_string())
}

fn print_result(result: &GenerateResult) {
    if result.written_paths.is_empty() {
        println!("No files generated.");
        return;
    }
    if result.dry_run {
        println!("Dry run complete. Planned files:");
    } else {
        println!("Generated files:");
    }
    for path in &result.written_paths {
        println!("  - {}", path.display());
    }
}

fn parse_command(args: Vec<String>) -> color_eyre::Result<Command> {
    if args.is_empty() {
        return Ok(Command::Generate(GenerateArgs {
            target: None,
            template: Template::Minimal,
            name: None,
            output: None,
            force: false,
            dry_run: false,
            install_deps: true,
        }));
    }

    if args[0] == "help" || args[0] == "--help" || args[0] == "-h" {
        return Ok(Command::Help);
    }

    if args[0] != "generate" {
        return Err(color_eyre::eyre::eyre!(
            "unknown command `{}`. Use `cli help`.",
            args[0]
        ));
    }

    let mut parsed = GenerateArgs {
        target: None,
        template: Template::Minimal,
        name: None,
        output: None,
        force: false,
        dry_run: false,
        install_deps: true,
    };

    let mut index = 1usize;
    while index < args.len() {
        match args[index].as_str() {
            "--target" => {
                let value = next_value(&args, index, "--target")?;
                parsed.target = Some(Target::parse(value)?);
                index += 2;
            }
            "--template" => {
                let value = next_value(&args, index, "--template")?;
                parsed.template = Template::parse(value)?;
                index += 2;
            }
            "--name" => {
                let value = next_value(&args, index, "--name")?;
                parsed.name = Some(value.to_string());
                index += 2;
            }
            "--output" => {
                let value = next_value(&args, index, "--output")?;
                parsed.output = Some(PathBuf::from(value));
                index += 2;
            }
            "--force" => {
                parsed.force = true;
                index += 1;
            }
            "--dry-run" => {
                parsed.dry_run = true;
                index += 1;
            }
            "--no-install" => {
                parsed.install_deps = false;
                index += 1;
            }
            "--install" => {
                parsed.install_deps = true;
                index += 1;
            }
            unknown => {
                return Err(color_eyre::eyre::eyre!(
                    "unknown flag `{unknown}`. Use `cli help`."
                ));
            }
        }
    }

    Ok(Command::Generate(parsed))
}

fn next_value<'a>(args: &'a [String], index: usize, flag: &str) -> color_eyre::Result<&'a str> {
    args.get(index + 1)
        .map(String::as_str)
        .ok_or_else(|| color_eyre::eyre::eyre!("missing value for {flag}"))
}

fn print_help() {
    println!(
        "Usage:
  cli generate [--target opencode|pi] [--template minimal] [--name NAME] [--output PATH] [--force] [--dry-run] [--no-install]

Notes:
  - If --target/--name are not provided, interactive ratatui wizard starts.
  - If --output is omitted, defaults to ./<name>.
  - Dependencies are installed by default (use --no-install to skip).
  - `minimal` is the initial template for both targets."
    );
}
