use std::env;
use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::{DefaultTerminal, Frame};

use crate::generator::{GenerateOptions, Target, Template};

enum Step {
    SelectTarget,
    SelectTemplate,
    EditName,
    EditOutput,
    Confirm,
    Done,
}

pub struct WizardOutput {
    pub options: GenerateOptions,
    pub install_deps: bool,
}

struct WizardState {
    step: Step,
    target_index: usize,
    template_index: usize,
    project_name: String,
    output_path: String,
    install_deps: bool,
    cwd: PathBuf,
}

impl WizardState {
    fn apply_key(&mut self, key: KeyCode) -> color_eyre::Result<bool> {
        match self.step {
            Step::SelectTarget => match key {
                KeyCode::Up | KeyCode::Left => {
                    self.target_index = self.target_index.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Right => {
                    self.target_index = (self.target_index + 1).min(1);
                }
                KeyCode::Enter => self.step = Step::SelectTemplate,
                _ => {}
            },
            Step::SelectTemplate => match key {
                KeyCode::Up | KeyCode::Left => {
                    self.template_index = 0;
                }
                KeyCode::Down | KeyCode::Right => {
                    self.template_index = 0;
                }
                KeyCode::Backspace => self.step = Step::SelectTarget,
                KeyCode::Enter => self.step = Step::EditName,
                _ => {}
            },
            Step::EditName => match key {
                KeyCode::Char(ch) => self.project_name.push(ch),
                KeyCode::Backspace => {
                    self.project_name.pop();
                }
                KeyCode::Enter => {
                    if self.project_name.trim().is_empty() {
                        return Err(color_eyre::eyre::eyre!("project name cannot be empty"));
                    }
                    self.refresh_default_output();
                    self.step = Step::EditOutput;
                }
                KeyCode::Esc => self.step = Step::SelectTemplate,
                _ => {}
            },
            Step::EditOutput => match key {
                KeyCode::Char(ch) => self.output_path.push(ch),
                KeyCode::Backspace => {
                    self.output_path.pop();
                }
                KeyCode::Enter => self.step = Step::Confirm,
                KeyCode::Esc => self.step = Step::EditName,
                _ => {}
            },
            Step::Confirm => match key {
                KeyCode::Char('b') => self.step = Step::EditOutput,
                KeyCode::Char('i') => {
                    self.install_deps = !self.install_deps;
                }
                KeyCode::Enter => {
                    self.step = Step::Done;
                    return Ok(true);
                }
                _ => {}
            },
            Step::Done => {}
        }
        Ok(false)
    }

    fn selected_target(&self) -> Target {
        if self.target_index == 0 {
            Target::OpenCode
        } else {
            Target::Pi
        }
    }

    fn selected_template(&self) -> Template {
        let _ = self.template_index;
        Template::Minimal
    }

    fn to_options(&self) -> WizardOutput {
        WizardOutput {
            options: GenerateOptions {
                target: self.selected_target(),
                template: self.selected_template(),
                project_name: self.project_name.trim().to_string(),
                output_dir: PathBuf::from(self.output_path.trim()),
                sdk_dependency: String::new(),
                force: false,
                dry_run: false,
            },
            install_deps: self.install_deps,
        }
    }

    fn refresh_default_output(&mut self) {
        let current = self.output_path.trim();
        let expected_prefix = self.cwd.to_string_lossy().to_string();
        if current.is_empty() || current == expected_prefix || current.ends_with("/my-plugin") {
            self.output_path = self
                .cwd
                .join(self.project_name.trim())
                .to_string_lossy()
                .to_string();
        }
    }
}

pub fn run_generate_wizard(terminal: &mut DefaultTerminal) -> color_eyre::Result<WizardOutput> {
    let cwd = env::current_dir().map_err(|e| color_eyre::eyre::eyre!(e))?;
    let mut state = WizardState {
        step: Step::SelectTarget,
        target_index: 0,
        template_index: 0,
        project_name: "my-plugin".to_string(),
        output_path: cwd.join("my-plugin").to_string_lossy().to_string(),
        install_deps: true,
        cwd,
    };

    loop {
        terminal.draw(|frame| render(frame, &state))?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if key.code == KeyCode::Char('q') {
                return Err(color_eyre::eyre::eyre!("generation cancelled"));
            }
            if state.apply_key(key.code)? {
                return Ok(state.to_options());
            }
        }
    }
}

fn render(frame: &mut Frame, state: &WizardState) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(10)]).split(frame.area());
    frame.render_widget(
        Paragraph::new("Plugin Scaffold Generator  |  q: quit")
            .block(Block::default().borders(Borders::ALL)),
        chunks[0],
    );

    match state.step {
        Step::SelectTarget => render_target(frame, chunks[1], state),
        Step::SelectTemplate => render_template(frame, chunks[1], state),
        Step::EditName => render_name(frame, chunks[1], state),
        Step::EditOutput => render_output(frame, chunks[1], state),
        Step::Confirm => render_confirm(frame, chunks[1], state),
        Step::Done => {}
    }
}

fn render_target(frame: &mut Frame, area: ratatui::layout::Rect, state: &WizardState) {
    let items = ["opencode", "pi"]
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            if idx == state.target_index {
                ListItem::new(*value).style(Style::default().add_modifier(Modifier::BOLD))
            } else {
                ListItem::new(*value)
            }
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select target (arrow keys + Enter)"),
        ),
        area,
    );
}

fn render_template(frame: &mut Frame, area: ratatui::layout::Rect, state: &WizardState) {
    let item = if state.template_index == 0 {
        ListItem::new("minimal").style(Style::default().add_modifier(Modifier::BOLD))
    } else {
        ListItem::new("minimal")
    };
    frame.render_widget(
        List::new(vec![item]).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select template (Enter to continue, Backspace to go back)"),
        ),
        area,
    );
}

fn render_name(frame: &mut Frame, area: ratatui::layout::Rect, state: &WizardState) {
    frame.render_widget(
        Paragraph::new(state.project_name.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Project name (type + Enter, Esc to go back)"),
        ),
        area,
    );
}

fn render_output(frame: &mut Frame, area: ratatui::layout::Rect, state: &WizardState) {
    frame.render_widget(
        Paragraph::new(state.output_path.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Output directory (type + Enter, Esc to go back)"),
        ),
        area,
    );
}

fn render_confirm(frame: &mut Frame, area: ratatui::layout::Rect, state: &WizardState) {
    let preview = format!(
        "target: {}\ntemplate: {}\nname: {}\noutput: {}\ninstall dependencies: {}\n\nEnter: generate | b: back | i: toggle install",
        state.selected_target(),
        state.selected_template(),
        state.project_name.trim(),
        state.output_path.trim(),
        if state.install_deps { "yes" } else { "no" },
    );
    frame.render_widget(
        Paragraph::new(preview).block(Block::default().borders(Borders::ALL).title("Confirm")),
        area,
    );
}
