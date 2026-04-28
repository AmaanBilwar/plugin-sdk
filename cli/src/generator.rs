use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::templates::{self, RenderedTemplateFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    OpenCode,
    Pi,
}

impl Target {
    pub fn parse(raw: &str) -> color_eyre::Result<Self> {
        match raw {
            "opencode" => Ok(Self::OpenCode),
            "pi" => Ok(Self::Pi),
            _ => Err(color_eyre::eyre::eyre!(
                "invalid target `{raw}`. Use `opencode` or `pi`."
            )),
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::OpenCode => write!(f, "opencode"),
            Target::Pi => write!(f, "pi"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    Minimal,
}

impl Template {
    pub fn parse(raw: &str) -> color_eyre::Result<Self> {
        match raw {
            "minimal" => Ok(Self::Minimal),
            _ => Err(color_eyre::eyre::eyre!(
                "invalid template `{raw}`. Only `minimal` is supported right now."
            )),
        }
    }
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Template::Minimal => write!(f, "minimal"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub target: Target,
    pub template: Template,
    pub project_name: String,
    pub output_dir: PathBuf,
    pub sdk_dependency: String,
    pub force: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct GenerateResult {
    pub written_paths: Vec<PathBuf>,
    pub dry_run: bool,
}

pub fn generate(options: &GenerateOptions) -> color_eyre::Result<GenerateResult> {
    let files = templates::render(options)?;
    if !options.dry_run {
        validate_collisions(&options.output_dir, &files, options.force)?;
    }

    let mut written = Vec::with_capacity(files.len());
    for file in files {
        let absolute_path = options.output_dir.join(&file.relative_path);
        if !options.dry_run {
            write_file(&absolute_path, &file.contents)?;
        }
        written.push(absolute_path);
    }

    Ok(GenerateResult {
        written_paths: written,
        dry_run: options.dry_run,
    })
}

fn validate_collisions(
    output_dir: &Path,
    files: &[RenderedTemplateFile],
    force: bool,
) -> color_eyre::Result<()> {
    if force {
        return Ok(());
    }

    if let Some(existing) = files
        .iter()
        .map(|file| output_dir.join(&file.relative_path))
        .find(|path| path.exists())
    {
        return Err(color_eyre::eyre::eyre!(
            "refusing to overwrite existing file: {}. Re-run with --force.",
            existing.display()
        ));
    }

    Ok(())
}

fn write_file(path: &Path, contents: &str) -> color_eyre::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| color_eyre::eyre::eyre!(e))?;
    }
    fs::write(path, contents).map_err(|e| color_eyre::eyre::eyre!(e))?;
    Ok(())
}
