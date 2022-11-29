use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use log;
use std::{
    io::{self, Stdout},
    sync::mpsc::Receiver,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    // terminal::{enable_raw_mode,disable_raw_mode},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Tabs},
    Terminal,
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

// use crossterm::{
//     event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
//     execute,
//     terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
// };

use crate::comms::Comms;

use super::utils;

pub fn init_terminal(recv: Receiver<Comms>) {
    TerminalRunner::new(recv).run();
}

struct TerminalRunner {
    recv: Receiver<Comms>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    connected: Vec<String>,
    progress: u16,
}

impl TerminalRunner {
    fn new(recv: Receiver<Comms>) -> Self {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).expect("couldn't create a terminal");
        Self {
            recv,
            terminal,
            connected: Vec::new(),
            progress: 0,
        }
    }

    fn run(&mut self) {
        let _ = enable_raw_mode();

        tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
        tui_logger::set_default_level(log::LevelFilter::Trace);

        loop {
            if let Ok(comms) = self.recv.try_recv() {
                self.connected = comms.connected;
            }

            let _ = self.terminal.draw(|rect| {
                let size = rect.size();
                let top = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(20), Constraint::Percentage(50)].as_ref())
                    .split(size);
                // let body = Layout::default()
                //     .direction(Direction::Horizontal)
                //     .margin(1)
                //     .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                //     .split(size);

                let header = get_title();
                rect.render_widget(header, top[0]);

                let tui_w: TuiLoggerWidget = TuiLoggerWidget::default()
                    .block(
                        Block::default()
                            .title("Log Console")
                            .border_style(Style::default().fg(Color::White).bg(Color::Black))
                            .borders(Borders::ALL),
                    )
                    .output_separator('|')
                    .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
                    .output_level(Some(TuiLoggerLevelOutput::Long))
                    .output_target(false)
                    .output_file(false)
                    .output_line(false)
                    .style(Style::default().fg(Color::White).bg(Color::Black));

                rect.render_widget(tui_w, top[1]);

                // let conn_w = get_connected(&self.connected);

                // rect.render_widget(conn_w, body[0]);

                // let spin = get_bar(&mut self.progress);
                // rect.render_widget(spin, body[1]);
            });

            utils::wait(utils::DEFAULT_WAIT);
        }

        // let = disable_raw_mode();
    }
}

fn get_title() -> Paragraph<'static> {
    let text = Spans::from(Span::from("Monarch Proxichat Server"));

    Paragraph::new(text)
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Proxichat")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().add_modifier(Modifier::BOLD)),
        )
}

fn get_nav() -> Tabs<'static> {
    let style = Style::default()
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::UNDERLINED);
    let text = vec![
        Spans::from(Span::styled("none", style)),
        Spans::from(Span::styled("none2", style)),
    ];

    Tabs::new(text)
        .style(Style::default().fg(Color::LightCyan))
        .divider("|")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Navigation")
                .border_type(BorderType::Rounded),
        )
}

fn get_connected(connected: &[String]) -> Paragraph {
    let text: Vec<Spans> = connected
        .iter()
        .map(|c| {
            let s = Span::styled(c, Style::default().add_modifier(Modifier::BOLD));
            Spans::from(s)
        })
        .collect();

    Paragraph::new(text)
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Connections")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().add_modifier(Modifier::BOLD)),
        )
}

fn get_bar(progress: &mut u16) -> Gauge {
    if *progress >= 100_u16 {
        *progress = 0;
    } else {
        *progress += 1;
    }

    Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Moving Thing")
                .border_type(BorderType::Rounded),
        )
        .gauge_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC),
        )
        .percent(*progress)
}
