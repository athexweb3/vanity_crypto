use crate::app::{App, AppState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::sync::atomic::Ordering;

pub fn ui(f: &mut Frame, app: &mut App) {
    // Common Layout: Header (3) | Content (Min) | Spacer (1) | Footer/Help (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Content
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Footer/Help
        ])
        .split(f.area());

    render_header(f, chunks[0]);

    match app.state {
        AppState::Config => {
            render_config(f, app, chunks[1]);
            render_config_footer(f, chunks[3]);
        }
        AppState::Searching | AppState::Finished => {
            render_searching_body(f, app, chunks[1]);
            render_search_footer(f, chunks[3]);
        }
    }
}

fn render_header(f: &mut Frame, area: ratatui::layout::Rect) {
    let title = Paragraph::new(Text::from(vec![Line::from(vec![
        Span::styled(
            " VANITY ",
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::styled(
            " CRYPTO ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ])]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn render_config_footer(f: &mut Frame, area: ratatui::layout::Rect) {
    let help = Paragraph::new("Tab: Next Field | Enter: Select/Start | Esc: Quit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(help, area);
}

fn render_search_footer(f: &mut Frame, area: ratatui::layout::Rect) {
    let footer = Paragraph::new("Press 'q' to quit (WARNING: Progress will be lost)")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(footer, area);
}

fn render_config(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    // Top-Left aligned with padding
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2), // Left Padding
            Constraint::Min(1),    // Content
        ])
        .split(area);

    let content_area = layout[1];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top Padding
            Constraint::Length(2), // Subtitle
            Constraint::Length(2), // Chain (Index 2 in chunks)
            Constraint::Length(2), // Network (Index 3)
            Constraint::Length(2), // Type  (Index 4)
            Constraint::Length(2), // Prefix (5)
            Constraint::Length(2), // Suffix (6)
            Constraint::Length(2), // Options (7)
            Constraint::Length(2), // Button (8)
        ])
        .split(content_area);

    // Sub-title
    let title = Paragraph::new("Enter Search Patterns")
        .alignment(Alignment::Left)
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title, chunks[1]);

    // Helpers
    let active_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let inactive_style = Style::default().fg(Color::Gray);
    let cursor_symbol = |idx| {
        if app.input_focus_index == idx {
            "> "
        } else {
            "  "
        }
    };
    let style_for = |idx| {
        if app.input_focus_index == idx {
            active_style
        } else {
            inactive_style
        }
    };

    // 0. Chain
    let chain_str = format!("{:?}", app.chain);
    let chain_text = vec![
        Span::styled(cursor_symbol(0), style_for(0)),
        Span::styled("Chain  : ", style_for(0)),
        Span::styled(
            format!("< {} >", chain_str),
            if app.input_focus_index == 0 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            },
        ),
    ];
    f.render_widget(Paragraph::new(Line::from(chain_text)), chunks[2]);

    // 1. Network
    let network_str = if app.chain == crate::app::Chain::Bitcoin {
        format!("{:?}", app.network)
    } else {
        "N/A".to_string()
    };
    let network_style = if app.chain == crate::app::Chain::Bitcoin {
        style_for(1)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let network_text = vec![
        Span::styled(cursor_symbol(1), network_style),
        Span::styled("Network: ", network_style),
        Span::styled(
            format!("< {} >", network_str),
            if app.input_focus_index == 1 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
    ];
    f.render_widget(Paragraph::new(Line::from(network_text)), chunks[3]);

    // 2. Type
    let type_str = if app.chain == crate::app::Chain::Bitcoin {
        format!("{:?}", app.btc_type)
    } else if app.chain == crate::app::Chain::Ton {
        format!("{:?}", app.ton_version)
    } else if app.chain == crate::app::Chain::Cosmos {
        if app.hrp.is_empty() { "..." } else { &app.hrp }.to_string()
    } else {
        "N/A".to_string()
    };
    let type_style = if app.chain == crate::app::Chain::Bitcoin
        || app.chain == crate::app::Chain::Ton
        || app.chain == crate::app::Chain::Cosmos
    {
        style_for(2)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let type_label = if app.chain == crate::app::Chain::Cosmos {
        "HRP    : "
    } else {
        "Type   : "
    };
    let type_text = vec![
        Span::styled(cursor_symbol(2), type_style),
        Span::styled(type_label, type_style),
        Span::styled(
            format!("< {} >", type_str),
            if app.input_focus_index == 2 {
                Style::default().fg(Color::Cyan)
            } else if app.chain == crate::app::Chain::Bitcoin
                || app.chain == crate::app::Chain::Ton
                || app.chain == crate::app::Chain::Cosmos
            {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
    ];
    f.render_widget(Paragraph::new(Line::from(type_text)), chunks[4]);

    // 3. Prefix
    let prefix_val = if app.prefix.is_empty() {
        "..."
    } else {
        &app.prefix
    };
    let prefix_text = vec![
        Span::styled(cursor_symbol(3), style_for(3)),
        Span::styled("Prefix : ", style_for(3)),
        Span::raw(prefix_val),
    ];
    let prefix_p = Paragraph::new(Line::from(prefix_text));
    f.render_widget(prefix_p, chunks[5]);

    // 4. Suffix
    let suffix_val = if app.suffix.is_empty() {
        "..."
    } else {
        &app.suffix
    };
    let suffix_text = vec![
        Span::styled(cursor_symbol(4), style_for(4)),
        Span::styled("Suffix : ", style_for(4)),
        Span::raw(suffix_val),
    ];
    let suffix_p = Paragraph::new(Line::from(suffix_text));
    f.render_widget(suffix_p, chunks[6]);

    // 5. Options
    let check = if app.case_sensitive { "[x]" } else { "[ ]" };
    let opts_text = vec![
        Span::styled(cursor_symbol(5), style_for(5)),
        Span::styled("Options: ", style_for(5)),
        Span::raw(format!("{} Case Sensitive", check)),
    ];
    let opts_p = Paragraph::new(Line::from(opts_text));
    f.render_widget(opts_p, chunks[7]);

    // 6. Button
    let btn_style = if app.input_focus_index == 6 {
        Style::default()
            .bg(Color::Green)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let btn_label = if app.input_focus_index == 6 {
        format!("{}[ START ENGINE ] (or Ctrl+Enter)", cursor_symbol(6))
    } else {
        "  [ START ENGINE ] (or Ctrl+Enter)".to_string()
    };
    let btn_p = Paragraph::new(btn_label).style(btn_style);
    f.render_widget(btn_p, chunks[8]);
}

fn render_searching_body(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let attempts = app.attempts.load(Ordering::Relaxed);
    let speed = app.rate_per_second;
    let elapsed = if let Some(start) = app.start_time {
        start.elapsed().as_secs()
    } else {
        0
    };

    if let Some(found) = &app.found_address {
        // Success View - Split into Result and Safety Warning
        let result_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(6),
                Constraint::Length(8), // Safety Warning
            ])
            .split(area);

        // Explicitly format the private key to be copy-friendly
        // Split long lines
        let pk_str = &found.1; // Private Key is index 1

        let pk_lines = if pk_str.len() > 60 {
            // Split in half essentially
            let mid = pk_str.len() / 2;
            vec![
                Line::from(Span::raw(&pk_str[..mid])),
                Line::from(Span::raw(&pk_str[mid..])),
            ]
        } else {
            vec![Line::from(pk_str.as_str())]
        };

        let mut success_text = vec![
            Line::from(vec![
                Span::styled("Address: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &found.0,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ), // Address is index 0
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Private Key:",
                Style::default().fg(Color::Gray),
            )),
        ];
        success_text.extend(pk_lines);

        // Add footer note about finding it in stdout
        success_text.push(Line::from(""));
        success_text.push(Line::from(Span::styled(
            "(Press 'q' to copy single-line key from terminal)",
            Style::default().fg(Color::DarkGray),
        )));

        let success_block = Paragraph::new(success_text)
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center)
            // .wrap(ratatui::widgets::Wrap { trim: true }) // DISABLED WRAPPING to prevent auto-breaks
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🎉 FOUND MATCH! 🎉 ")
                    .border_style(Style::default().fg(Color::Green)),
            );
        f.render_widget(success_block, result_chunks[0]);

        let warning_text = vec![
            Line::from(Span::styled(
                "⚠️  SECURITY WARNING ⚠️",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("1. This tool is offline, but your computer usage leaves traces."),
            Line::from("2. Send a SMALL test transaction to this address first."),
            Line::from("3. Verify you can access the funds before transferring large amounts."),
            Line::from("4. NEVER share the private key with anyone."),
        ];

        let warning_block = Paragraph::new(warning_text)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Safety First ")
                    .border_style(Style::default().fg(Color::Red)),
            );
        f.render_widget(warning_block, result_chunks[1]);
    } else {
        // Search View
        let stats_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Config Block
        let cs_text = if app.case_sensitive { "Yes" } else { "No" };
        let config_text = vec![
            Line::from(vec![
                Span::raw("Prefix : "),
                Span::styled(&app.prefix, Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::raw("Suffix : "),
                Span::styled(&app.suffix, Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::raw("Case   : "),
                Span::styled(cs_text, Style::default().fg(Color::Magenta)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Searching...",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::RAPID_BLINK),
            )]),
        ];

        let config_block = Paragraph::new(config_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Configuration "),
            )
            .alignment(Alignment::Left);
        f.render_widget(config_block, stats_chunks[0]);

        // Stats Block
        let stats_text = vec![
            Line::from(vec![
                Span::raw("Attempts : "),
                Span::styled(
                    format!("{}", attempts),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Speed    : "),
                Span::styled(
                    format!("{} keys/s", speed),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::raw("Time     : "),
                Span::styled(format!("{}s", elapsed), Style::default().fg(Color::Gray)),
            ]),
        ];

        let stats = Paragraph::new(stats_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Real-time Statistics "),
            )
            .alignment(Alignment::Left);

        f.render_widget(stats, stats_chunks[1]);
    }
}
