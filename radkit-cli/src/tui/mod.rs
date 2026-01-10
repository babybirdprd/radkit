pub mod app;
pub mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

#[derive(Debug)]
pub enum TuiAction {
    None,
    CreateAgent {
        name: String,
        template: String,
        provider: String,
    },
    AddTool {
        name: String,
        template: String,
    },
}

pub fn run() -> Result<TuiAction> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Main loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
        return Ok(TuiAction::None);
    }

    // Determine action from app state
    // We need to check if we exited with a "Done" state
    // The current `App` logic just sets `should_quit`.
    // We need to inspect `current_screen` or some other flag to know WHY we quit.
    // Let's infer from state for now, but better to add `exit_action` field to App later.
    // For now, I'll access the public fields of App.

    use app::{CurrentScreen, WizardStep, ToolWizardStep};

    match app.current_screen {
        CurrentScreen::CreateWizard(WizardStep::Confirmation) => {
            // If we are in Confirmation and quit, we assume the user pressed Enter which triggered quit
            // We should double check if `should_quit` was set by Enter or Quit.
            // But handle_input sets should_quit on Enter in Confirmation step.
            Ok(TuiAction::CreateAgent {
                name: app.project_name,
                template: app.templates[app.wizard_template_idx].clone(),
                provider: app.providers[app.wizard_provider_idx].clone(),
            })
        },
        CurrentScreen::ToolWizard(ToolWizardStep::Confirmation) => {
             Ok(TuiAction::AddTool {
                name: app.new_tool_name,
                template: app.selected_tool_template,
             })
        },
        _ => Ok(TuiAction::None),
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            app.handle_input(key);
            if app.should_quit {
                return Ok(());
            }
        }
    }
}
