use crossterm::event::KeyEvent;
use tui_input::Input;

#[derive(PartialEq)]
pub enum CurrentScreen {
    Home,
    CreateWizard(WizardStep),
    Dashboard(DashboardTab),
    ToolWizard(ToolWizardStep),
}

#[derive(PartialEq, Clone, Copy)]
pub enum WizardStep {
    NameInput,
    TemplateSelection,
    ProviderSelection,
    Confirmation,
}

#[derive(PartialEq, Clone, Copy)]
pub enum ToolWizardStep {
    TemplateSelection,
    NameInput,
    Confirmation,
}

#[derive(PartialEq, Clone, Copy)]
pub enum DashboardTab {
    Overview,
    Tools,
    Skills,
    Providers,
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub should_quit: bool,
    pub _is_in_project: bool,

    // Wizard State (Create Agent)
    pub wizard_input: Input,
    pub wizard_template_idx: usize,
    pub wizard_provider_idx: usize,
    pub templates: Vec<String>,
    pub providers: Vec<String>,
    pub project_name: String,

    // Tool Wizard State
    pub tool_templates: Vec<String>,
    pub tool_template_idx: usize,
    pub tool_name_input: Input,
    pub new_tool_name: String,
    pub selected_tool_template: String,

    // Dashboard State
    pub dashboard_tab: DashboardTab,
    pub tools_list: Vec<String>,
    pub skills_list: Vec<String>,
    pub current_provider: String,
}

impl App {
    pub fn new() -> App {
        let is_in_project = std::path::Path::new("Cargo.toml").exists();

        let mut app = App {
            current_screen: if is_in_project {
                CurrentScreen::Dashboard(DashboardTab::Overview)
            } else {
                CurrentScreen::Home
            },
            should_quit: false,
            _is_in_project: is_in_project,

            wizard_input: Input::default(),
            wizard_template_idx: 0,
            wizard_provider_idx: 0,
            templates: vec![],
            providers: vec![
                "Gemini".to_string(),
                "OpenAI".to_string(),
                "Anthropic".to_string(),
                "DeepSeek".to_string(),
                "Grok".to_string(),
                "OpenRouter".to_string()
            ],
            project_name: String::new(),

            tool_templates: vec![
                "Blank Tool".to_string(),
                "Calculator".to_string(),
                "Web Search".to_string(),
                "File Reader".to_string(),
                "File Writer".to_string(),
                "Command Runner".to_string(),
                "HTTP Request".to_string(),
                "System Control".to_string(),
            ],
            tool_template_idx: 0,
            tool_name_input: Input::default(),
            new_tool_name: String::new(),
            selected_tool_template: String::new(),

            dashboard_tab: DashboardTab::Overview,
            tools_list: vec![],
            skills_list: vec![],
            current_provider: "Unknown".to_string(),
        };

        if !is_in_project {
             // Populate templates if we can find them, or hardcode for now
             // In a real scenario we'd use include_dir to list them
             app.templates = vec![
                 "simple-agent".to_string(),
                 "interactive-agent".to_string(),
                 "advanced-agent".to_string(),
                 "live-agent".to_string(),
                 "rag-agent".to_string(),
                 "a2a-agent".to_string(),
             ];
        } else {
            // Load project info
            app.refresh_project_data();
        }

        app
    }

    pub fn refresh_project_data(&mut self) {
        // Simple scan for tools and skills
        if let Ok(entries) = std::fs::read_dir("src/tools") {
            self.tools_list = entries.filter_map(|e| {
                let path = e.ok()?.path();
                if path.extension()?.to_str()? == "rs" && path.file_stem()?.to_str()? != "mod" {
                     Some(path.file_stem()?.to_string_lossy().to_string())
                } else {
                    None
                }
            }).collect();
        }

        if let Ok(entries) = std::fs::read_dir("src/skills") {
            self.skills_list = entries.filter_map(|e| {
                let path = e.ok()?.path();
                if path.extension()?.to_str()? == "rs" && path.file_stem()?.to_str()? != "mod" {
                     Some(path.file_stem()?.to_string_lossy().to_string())
                } else {
                    None
                }
            }).collect();
        }

        // Try to guess provider from main.rs (very basic check)
        if let Ok(content) = std::fs::read_to_string("src/main.rs") {
             if content.contains("GeminiLlm") { self.current_provider = "Gemini".to_string(); }
             else if content.contains("OpenAILlm") { self.current_provider = "OpenAI".to_string(); }
             else if content.contains("AnthropicLlm") { self.current_provider = "Anthropic".to_string(); }
             else if content.contains("DeepSeekLlm") { self.current_provider = "DeepSeek".to_string(); }
             else if content.contains("GrokLlm") { self.current_provider = "Grok".to_string(); }
             else if content.contains("OpenRouterLlm") { self.current_provider = "OpenRouter".to_string(); }
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) {
        use crossterm::event::KeyCode;

        match &self.current_screen {
            CurrentScreen::Home => {
                match key.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Char('c') => {
                        self.current_screen = CurrentScreen::CreateWizard(WizardStep::NameInput);
                    },
                    _ => {}
                }
            },
            CurrentScreen::CreateWizard(step) => self.handle_wizard_input(key, *step),
            CurrentScreen::Dashboard(_) => self.handle_dashboard_input(key),
            CurrentScreen::ToolWizard(step) => self.handle_tool_wizard_input(key, *step),
        }
    }

    fn handle_wizard_input(&mut self, key: KeyEvent, step: WizardStep) {
         use crossterm::event::KeyCode;
         match step {
             WizardStep::NameInput => {
                 match key.code {
                     KeyCode::Enter => {
                         self.project_name = self.wizard_input.value().to_string();
                         if !self.project_name.is_empty() {
                            self.current_screen = CurrentScreen::CreateWizard(WizardStep::TemplateSelection);
                         }
                     },
                     KeyCode::Esc => self.current_screen = CurrentScreen::Home,
                     _ => {
                         tui_input::backend::crossterm::EventHandler::handle_event(&mut self.wizard_input, &crossterm::event::Event::Key(key));
                     }
                 }
             },
             WizardStep::TemplateSelection => {
                 match key.code {
                     KeyCode::Up => if self.wizard_template_idx > 0 { self.wizard_template_idx -= 1; },
                     KeyCode::Down => if self.wizard_template_idx < self.templates.len() - 1 { self.wizard_template_idx += 1; },
                     KeyCode::Enter => self.current_screen = CurrentScreen::CreateWizard(WizardStep::ProviderSelection),
                     KeyCode::Esc => self.current_screen = CurrentScreen::CreateWizard(WizardStep::NameInput),
                     _ => {}
                 }
             },
             WizardStep::ProviderSelection => {
                 match key.code {
                     KeyCode::Up => if self.wizard_provider_idx > 0 { self.wizard_provider_idx -= 1; },
                     KeyCode::Down => if self.wizard_provider_idx < self.providers.len() - 1 { self.wizard_provider_idx += 1; },
                     KeyCode::Enter => {
                         // Perform Creation Logic Here? Or transition to Confirmation?
                         // For now, let's just create it and exit or go to confirmation.
                         // Calling crate::commands::create::create_agent might be tricky due to output to stdout.
                         // But we can try.
                         self.current_screen = CurrentScreen::CreateWizard(WizardStep::Confirmation);
                     },
                     KeyCode::Esc => self.current_screen = CurrentScreen::CreateWizard(WizardStep::TemplateSelection),
                     _ => {}
                 }
             },
             WizardStep::Confirmation => {
                 match key.code {
                     KeyCode::Enter => {
                         // Trigger actual creation
                         // We probably need to exit TUI to print logs or handle it gracefully?
                         // For "AMAZING DX", we should show a spinner or status.
                         // But for MVP, let's exit and run it, or run it and capture output.

                         // We will invoke the create command here.
                         // Since `create_agent` prints to stdout, it might mess up TUI.
                         // Ideally `create_agent` should take a reporter or return result silently.

                         // We'll set a flag or just do it and let the TUI cleanup handle it?
                         // No, better to suspend TUI.

                         self.should_quit = true; // For now exit to create
                         // In a real app we'd run creation and show progress.
                         // But we need to pass data back to main to run it.
                         // Or we can run it here if we suspend terminal.
                     },
                     KeyCode::Esc => self.current_screen = CurrentScreen::CreateWizard(WizardStep::ProviderSelection),
                     _ => {}
                 }
             }
         }
    }

    fn handle_dashboard_input(&mut self, key: KeyEvent) {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab | KeyCode::Right => {
                self.dashboard_tab = match self.dashboard_tab {
                    DashboardTab::Overview => DashboardTab::Tools,
                    DashboardTab::Tools => DashboardTab::Skills,
                    DashboardTab::Skills => DashboardTab::Providers,
                    DashboardTab::Providers => DashboardTab::Overview,
                }
            },
            KeyCode::Left => {
                self.dashboard_tab = match self.dashboard_tab {
                    DashboardTab::Overview => DashboardTab::Providers,
                    DashboardTab::Tools => DashboardTab::Overview,
                    DashboardTab::Skills => DashboardTab::Tools,
                    DashboardTab::Providers => DashboardTab::Skills,
                }
            },
            KeyCode::Char('a') => {
                // 'a' for Add Tool/Skill depending on tab
                if self.dashboard_tab == DashboardTab::Tools {
                    self.current_screen = CurrentScreen::ToolWizard(ToolWizardStep::TemplateSelection);
                }
            },
            _ => {}
        }
    }

    fn handle_tool_wizard_input(&mut self, key: KeyEvent, step: ToolWizardStep) {
         use crossterm::event::KeyCode;
         match step {
             ToolWizardStep::TemplateSelection => {
                 match key.code {
                     KeyCode::Up => if self.tool_template_idx > 0 { self.tool_template_idx -= 1; },
                     KeyCode::Down => if self.tool_template_idx < self.tool_templates.len() - 1 { self.tool_template_idx += 1; },
                     KeyCode::Enter => {
                         self.selected_tool_template = self.tool_templates[self.tool_template_idx].clone();
                         self.current_screen = CurrentScreen::ToolWizard(ToolWizardStep::NameInput);
                     },
                     KeyCode::Esc => self.current_screen = CurrentScreen::Dashboard(DashboardTab::Tools),
                     _ => {}
                 }
             },
             ToolWizardStep::NameInput => {
                 match key.code {
                     KeyCode::Enter => {
                         self.new_tool_name = self.tool_name_input.value().to_string();
                         if !self.new_tool_name.is_empty() {
                            self.current_screen = CurrentScreen::ToolWizard(ToolWizardStep::Confirmation);
                         }
                     },
                     KeyCode::Esc => self.current_screen = CurrentScreen::ToolWizard(ToolWizardStep::TemplateSelection),
                     _ => {
                         tui_input::backend::crossterm::EventHandler::handle_event(&mut self.tool_name_input, &crossterm::event::Event::Key(key));
                     }
                 }
             },
             ToolWizardStep::Confirmation => {
                 match key.code {
                     KeyCode::Enter => {
                         // We exit to run the command, or trigger it.
                         // We need a way to execute the action.
                         // For now, let's signal exit to run.
                         // But we want to stay in TUI?
                         // If we stay, we need to call `tool::add_tool` logic but modified to take args and not use interactive input.
                         // Since `tool::add_tool` uses `dialoguer`, it's blocking and interactive.
                         // We need to refactor `tool::add_tool` or make a non-interactive version.

                         // For the purpose of this task, I will implement a "Pending Action" system in App
                         // or just let it exit for now.
                         // "AMAZING DX" suggests it should just work.

                         self.should_quit = true;
                     },
                     KeyCode::Esc => self.current_screen = CurrentScreen::ToolWizard(ToolWizardStep::NameInput),
                     _ => {}
                 }
             }
         }
    }
}
