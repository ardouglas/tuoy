use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::layout::{Constraint, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Row, Table, TableState};
use tui::{backend::CrosstermBackend, Terminal};

use std::{
    io::{stdout, Write},
    sync::mpsc,
    thread,
    time::Duration,
};

mod net;

pub enum Event<I> {
    Input(I),
}

pub struct StatefulTable<'a> {
    state: TableState,
    items: Vec<Vec<&'a str>>,
}

impl<'a> StatefulTable<'a> {
    fn new(rows: Vec<&'a str>) -> StatefulTable<'a> {
        let mut items = Vec::new();
        for row in rows {
            if !row.starts_with('#') {
                let split = row.split_ascii_whitespace().collect::<Vec<&str>>();
                items.push(split);
            }
        }

        StatefulTable {
            state: TableState::default(),
            items,
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    let latest_obs_resp = net::get_latest_obs().await.unwrap();

    let body = latest_obs_resp.text().await.unwrap();
    let split = body.split('\n');
    let row_strs = split.collect::<Vec<&str>>();

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            if event::poll(Duration::from_millis(200)).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
        }
    });

    let mut table = StatefulTable::new(row_strs);

    // Input
    loop {
        terminal.draw(|mut f| {
            let rects = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(2)
                .split(f.size());

            let selected_style = Style::default().fg(Color::LightCyan);
            let normal_style = Style::default().fg(Color::White);

            let header = [
                "stn", "lat", "lon", "year", "mo", "day", "hr", "min", "wdir", "wspd", "gst",
                "wvht", "dpd", "apd", "mwd", "pres", "ptdy", "atmp", "wtmp", "dewp", "vis", "tide",
            ];
            let rows = table
                .items
                .iter()
                .map(|i| Row::StyledData(i.iter(), normal_style));
            let t = Table::new(header.iter(), rows)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Latest Observations"),
                )
                .highlight_style(selected_style)
                .widths(&[
                    Constraint::Percentage(7),
                    Constraint::Percentage(5),
                    Constraint::Percentage(5),
                    Constraint::Percentage(5),
                    Constraint::Percentage(3),
                    Constraint::Percentage(3),
                    Constraint::Percentage(3),
                    Constraint::Percentage(3),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                ]);
            f.render_stateful_widget(t, rects[0], &mut table.state);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Down => {
                    table.next();
                }
                KeyCode::Up => {
                    table.previous();
                }
                _ => {}
            },
        }
    }

    Ok(())
}
