use crossterm::{
    event::{self, Event as CEvent, KeyCode, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::layout::{Constraint, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Row, Table, TableState};
use tui::{backend::CrosstermBackend, Terminal};
use std::fs::File;
use std::{
    io::{stdout, Write},
    sync::mpsc,
    thread,
    time::Duration,
};
use crossterm::event::MouseEvent;
use roxmltree::Node;

mod net;
mod state;

pub enum Event<I,J> {
    Key(I),
    Mouse(J),
}

pub struct StatefulTable {
    state: TableState,
    items: Vec<Vec<String>>,
}

impl StatefulTable {
    fn new(rows: Vec<Vec<String>>) -> StatefulTable {

        StatefulTable {
            state: TableState::default(),
            items: rows,
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
    let active_stations_resp = net::get_active_stations().await.unwrap();

    let body = active_stations_resp.text().await.unwrap();
    let active_stations = xml_to_stations(body);
    //let mut rows = Vec::new();
    let rows: Vec<Vec<String>> = active_stations.iter().map(|a| a.to_row()).collect();

    //let split = body.split('\n');
    //let row_strs = split.collect::<Vec<&str>>();

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut evt_file = File::create("evt.log").unwrap();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            if event::poll(Duration::from_millis(200)).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    evt_file.write_all("key_event\n".as_bytes()).unwrap();
                    tx.send(Event::Key(key)).unwrap();
                } else if let CEvent::Mouse(mouse) = event::read().unwrap(){
                    evt_file.write_all("mouse_event\n".as_bytes()).unwrap();
                    tx.send(Event::Mouse(mouse)).unwrap();
                }
            }
        }
    });

    let mut table = StatefulTable::new(rows);

    // Input
    loop {
        terminal.draw(|mut f| {
            let rects = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(2)
                .split(f.size());

            let selected_style = Style::default().fg(Color::LightCyan);
            let normal_style = Style::default().fg(Color::White);

//            let header = [
//                "stn", "lat", "lon", "year", "mo", "day", "hr", "min", "wdir", "wspd", "gst",
//                "wvht", "dpd", "apd", "mwd", "pres", "ptdy", "atmp", "wtmp", "dewp", "vis", "tide",
//            ];
            let header = ["station", "name", "lat", "lon", "program", "kind", "met", "currents", "water quality", "dart"];
            let rows = table
                .items
                .iter()
                .map(|i| Row::StyledData(i.iter(), normal_style));
            let t = Table::new(header.iter(), rows)
                .block(
                    Block::default()
                        //.borders(Borders::ALL)
                        .title("Active Stations"),
                )
                .highlight_style(selected_style)
                .widths(&[
                    Constraint::Percentage(5),
                    Constraint::Percentage(30),
                    Constraint::Percentage(4),
                    Constraint::Percentage(4),
                    Constraint::Percentage(25),
                    Constraint::Percentage(10),
                    Constraint::Percentage(3),
                    Constraint::Percentage(3),
                    Constraint::Percentage(10),
                    Constraint::Percentage(3),
                ]);
            f.render_stateful_widget(t, rects[0], &mut table.state);
        })?;

        match rx.recv()? {
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
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
            Event::Mouse(event) => match event{
                MouseEvent::ScrollDown(_, _, _) => {
                    
                    table.previous();
                },
                MouseEvent::ScrollUp(_, _, _) => {
                    table.next();
                },
                _ => {},
            },
        }
    }

    Ok(())
}

fn xml_to_stations(body: String) -> Vec<ActiveStation>{
    let doc = roxmltree::Document::parse(&body).unwrap();
    let active_stations: Vec<ActiveStation> = doc.descendants()
        .filter(|n| n.tag_name().name() == "station")
        .map(|n| ActiveStation::from_node(n))
        .collect();
    active_stations
}

struct ActiveStation{
    id: String,
    name: String,
    lat: String,
    lon: String,
    program: String,
    kind: String,
    met: String,
    currents: String,
    water_quality: String,
    dart: String,
}

impl ActiveStation{
    fn from_node(node: Node)-> ActiveStation{
        ActiveStation{
            id: node.attribute("id").map_or(String::from("whew, no id? how'd that happen"), |a| a.to_owned()),
            name: node.attribute("name").map_or(String::from("whew, no name? how'd that happen"), |a| a.to_owned()),
            lat: node.attribute("lat").map(|a| a.to_owned()).unwrap(),
            lon: node.attribute("lon").map(|a| a.to_owned()).unwrap(),
            program: node.attribute("pgm").map_or(String::from("whew, no pgm? how'd that happen"), |a| a.to_owned()),
            kind: node.attribute("type").map_or(String::from("whew, no kind? how'd that happen"), |a| a.to_owned()),
            met: node.attribute("met").map_or(String::from("n"),|a| a.to_owned()),
            currents: node.attribute("currents").map_or(String::from("n"),|a| a.to_owned()),
            water_quality: node.attribute("waterquality").map_or(String::from("n"),|a| a.to_owned()),
            dart: node.attribute("dart").map_or(String::from("n"),|a| a.to_owned()),
        }
    }

    fn to_row(&self)->Vec<String>{
        let mut row = Vec::new();
        row.push(self.id.clone());
        row.push(self.name.clone());
        row.push(self.lat.to_string());
        row.push(self.lon.to_string());
        row.push(self.program.clone());
        row.push(self.kind.clone());
        row.push(self.met.clone());
        row.push(self.currents.clone());
        row.push(self.water_quality.clone());
        row.push(self.dart.clone());

        row
    }
}