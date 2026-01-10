use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

use crate::tui::app::{App, CurrentScreen, DashboardTab, ToolWizardStep, WizardStep};

pub fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    // Background
    let block = Block::default().style(Style::default().bg(Color::Reset));
    f.render_widget(block, size);

    // Main layout with footer for status bar
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(size);

    let screen_area = main_layout[0];
    let status_bar_area = main_layout[1];

    match &app.current_screen {
        CurrentScreen::Home => render_home(f, app, screen_area),
        CurrentScreen::CreateWizard(step) => render_create_wizard(f, app, *step, screen_area),
        CurrentScreen::Dashboard(tab) => render_dashboard(f, app, *tab, screen_area),
        CurrentScreen::ToolWizard(step) => render_tool_wizard(f, app, *step, screen_area),
    }

    // Status Bar
    let status_text = match &app.current_screen {
        CurrentScreen::Home => "c: Create Agent | q: Quit",
        CurrentScreen::CreateWizard(_) => "↑/↓: Select | Enter: Confirm | Esc: Back",
        CurrentScreen::Dashboard(_) => "Tab/←/→: Switch Tab | a: Add Item | q: Quit",
        CurrentScreen::ToolWizard(_) => "↑/↓: Select | Enter: Confirm | Esc: Back",
    };
    let status_bar = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(status_bar, status_bar_area);
}

fn render_home(f: &mut Frame, _app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(area);

    let title_text = vec![
        Line::from(vec![Span::styled("   ___           _ _    _ _   ", Style::default().fg(Color::Cyan))]),
        Line::from(vec![Span::styled("  / _ \\ __ _  __| | | _(_) |_ ", Style::default().fg(Color::Cyan))]),
        Line::from(vec![Span::styled(" / /_)/ _` |/ _` | |/ / | __|", Style::default().fg(Color::Cyan))]),
        Line::from(vec![Span::styled("/ ___/ (_| | (_| |   <| | |_ ", Style::default().fg(Color::Cyan))]),
        Line::from(vec![Span::styled("\\/    \\__,_|\\__,_|_|\\_\\_|\\__|", Style::default().fg(Color::Cyan))]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "Welcome to Radkit CLI",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )]),
    ];

    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(title, chunks[0]);

    let menu_text = vec![
        Line::from(Span::styled(
            "Press 'c' to Create a New Agent",
            Style::default().fg(Color::Green),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'q' to Quit",
            Style::default().fg(Color::Red),
        )),
    ];

    let menu = Paragraph::new(menu_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Menu"));

    f.render_widget(menu, centered_rect(60, 20, chunks[1]));
}

fn render_create_wizard(f: &mut Frame, app: &App, step: WizardStep, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let step_num = match step {
        WizardStep::NameInput => 1,
        WizardStep::TemplateSelection => 2,
        WizardStep::ProviderSelection => 3,
        WizardStep::Confirmation => 4,
    };

    let title_text = format!("Create New Agent Wizard - Step {}/4", step_num);
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Help footer
    let help_text = match step {
        WizardStep::NameInput => "Enter name, Press <Enter> to continue, <Esc> to cancel",
        WizardStep::TemplateSelection => "Use <Up/Down> to select, <Enter> to confirm, <Esc> to back",
        WizardStep::ProviderSelection => "Use <Up/Down> to select, <Enter> to confirm, <Esc> to back",
        WizardStep::Confirmation => "Press <Enter> to CREATE, <Esc> to back",
    };
    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);

    // Content
    let content_area = chunks[1];

    match step {
        WizardStep::NameInput => {
            let input_block = Paragraph::new(app.wizard_input.value())
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Project Name"));
            f.render_widget(input_block, centered_rect(50, 3, content_area));
        },
        WizardStep::TemplateSelection => {
            let items: Vec<ListItem> = app.templates
                .iter()
                .enumerate()
                .map(|(i, t)| {
                     let style = if i == app.wizard_template_idx {
                         Style::default().fg(Color::Black).bg(Color::Green)
                     } else {
                         Style::default().fg(Color::White)
                     };
                     ListItem::new(Line::from(t.as_str())).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Select Template"));
            f.render_widget(list, centered_rect(50, 50, content_area));
        },
        WizardStep::ProviderSelection => {
             let items: Vec<ListItem> = app.providers
                .iter()
                .enumerate()
                .map(|(i, t)| {
                     let style = if i == app.wizard_provider_idx {
                         Style::default().fg(Color::Black).bg(Color::Green)
                     } else {
                         Style::default().fg(Color::White)
                     };
                     ListItem::new(Line::from(t.as_str())).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Select Provider"));
            f.render_widget(list, centered_rect(50, 50, content_area));
        },
        WizardStep::Confirmation => {
            let text = vec![
                Line::from(vec![Span::raw("Project Name: "), Span::styled(&app.project_name, Style::default().fg(Color::Green))]),
                Line::from(vec![Span::raw("Template: "), Span::styled(&app.templates[app.wizard_template_idx], Style::default().fg(Color::Green))]),
                Line::from(vec![Span::raw("Provider: "), Span::styled(&app.providers[app.wizard_provider_idx], Style::default().fg(Color::Green))]),
                Line::from(""),
                Line::from(Span::styled("Ready to create? Press Enter.", Style::default().add_modifier(Modifier::BOLD))),
            ];
             let p = Paragraph::new(text)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Confirmation"));
            f.render_widget(p, centered_rect(60, 40, content_area));
        }
    }
}

fn render_dashboard(f: &mut Frame, app: &App, tab: DashboardTab, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    // Tabs
    let titles: Vec<Line> = vec!["Overview", "Tools", "Skills", "Providers"]
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::styled(rest, Style::default().fg(Color::Green)),
            ])
        })
        .collect();

    let tab_index = match tab {
        DashboardTab::Overview => 0,
        DashboardTab::Tools => 1,
        DashboardTab::Skills => 2,
        DashboardTab::Providers => 3,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Radkit Dashboard"))
        .select(tab_index)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray));
    f.render_widget(tabs, chunks[0]);

    // Footer
    let footer = Paragraph::new("Dashboard Mode")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);

    // Content
    match tab {
        DashboardTab::Overview => {
            let info_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ])
                .split(chunks[1]);

            let provider_block = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(&app.current_provider, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            ])
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Provider"));

            let tools_block = Paragraph::new(vec![
                 Line::from(""),
                 Line::from(Span::styled(app.tools_list.len().to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            ])
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Tools"));

            let skills_block = Paragraph::new(vec![
                 Line::from(""),
                 Line::from(Span::styled(app.skills_list.len().to_string(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))),
            ])
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Skills"));

            f.render_widget(provider_block, info_chunks[0]);
            f.render_widget(tools_block, info_chunks[1]);
            f.render_widget(skills_block, info_chunks[2]);
        },
        DashboardTab::Tools => {
            let mut items: Vec<ListItem> = app.tools_list
                .iter()
                .map(|t| ListItem::new(Line::from(vec![Span::raw("- "), Span::styled(t, Style::default().fg(Color::Green))])))
                .collect();

            if items.is_empty() {
                items.push(ListItem::new(Line::from(Span::styled("(No tools configured)", Style::default().fg(Color::DarkGray)))));
            }

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Active Tools (Press 'a' to Add)"));
             f.render_widget(list, chunks[1]);
        },
        DashboardTab::Skills => {
            let items: Vec<ListItem> = app.skills_list
                .iter()
                .map(|t| ListItem::new(Line::from(vec![Span::raw("- "), Span::styled(t, Style::default().fg(Color::Blue))])))
                .collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Active Skills"));
             f.render_widget(list, chunks[1]);
        },
        DashboardTab::Providers => {
            let p = Paragraph::new("Provider Management coming soon to TUI.\nUse 'radkit provider' commands for now.")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(p, centered_rect(50, 20, chunks[1]));
        }
    }
}

fn render_tool_wizard(f: &mut Frame, app: &App, step: ToolWizardStep, area: Rect) {
    let centered = centered_rect(60, 40, area);
    f.render_widget(Block::default().borders(Borders::ALL).title("Add Tool Wizard"), centered);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(0)])
        .split(centered);

    match step {
        ToolWizardStep::TemplateSelection => {
             let items: Vec<ListItem> = app.tool_templates
                .iter()
                .enumerate()
                .map(|(i, t)| {
                     let style = if i == app.tool_template_idx {
                         Style::default().fg(Color::Black).bg(Color::Green)
                     } else {
                         Style::default().fg(Color::White)
                     };
                     ListItem::new(Line::from(t.as_str())).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Choose Template"));
            f.render_widget(list, chunks[0]);
        },
        ToolWizardStep::NameInput => {
             let input = Paragraph::new(app.tool_name_input.value())
                .block(Block::default().borders(Borders::ALL).title("Tool Name (Snake Case)"));
             f.render_widget(input, chunks[0]);
        },
        ToolWizardStep::Confirmation => {
             let text = vec![
                Line::from(vec![Span::raw("Template: "), Span::styled(&app.selected_tool_template, Style::default().fg(Color::Green))]),
                Line::from(vec![Span::raw("Name: "), Span::styled(&app.new_tool_name, Style::default().fg(Color::Green))]),
                Line::from(""),
                Line::from(Span::styled("Press Enter to Add Tool", Style::default().add_modifier(Modifier::BOLD))),
            ];
             let p = Paragraph::new(text)
                .alignment(Alignment::Center);
             f.render_widget(p, chunks[0]);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
