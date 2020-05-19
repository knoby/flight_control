use gilrs::{Axis, Button, EventType, Gilrs};

use serial::prelude::*;

use std::io::Write;

use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge},
    Terminal,
};

// Local Modules
mod util;
use util::event::{Event, Events};

#[derive(Debug)]
struct App {}

impl App {
    /// Create a new Instance of the application
    fn new() -> Self {
        App {}
    }

    /// Update the Input State and and read/send data to serial
    fn update(&mut self, gamepad: &mut Gilrs) {}

    /// Draw the status of the app to the screen
    fn draw<T>(&mut self, terminal: &mut Terminal<TermionBackend<T>>) -> Result<(), std::io::Error>
    where
        T: Write,
    {
        terminal.draw(|mut f| {
            let size = f.size();
            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, size);
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Gamepad Input
    let mut gilrs = Gilrs::new().map_err(|_| format!("Unable to init gamepad system"))?;

    // Create Serial Connection
    let mut port = serial::open("/dev/ttyACM0")
        .and_then(|mut port| {
            port.reconfigure(&|settings| {
                settings.set_baud_rate(serial::Baud9600).unwrap();
                settings.set_char_size(serial::Bits8);
                settings.set_parity(serial::ParityNone);
                settings.set_stop_bits(serial::Stop1);
                settings.set_flow_control(serial::FlowNone);
                Ok(())
            })?;
            port.set_timeout(std::time::Duration::from_millis(10))?;
            Ok(port)
        })
        .map_err(|_| format!("Unable to init serial communication"));

    // Create new Window
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    let mut app = App::new();

    loop {
        match events.next()? {
            Event::Input(input) => {
                if input == Key::Char('q') {
                    break;
                }
            }
            Event::Tick => {
                app.update(&mut gilrs);
            }
        }

        app.draw(&mut terminal)?;
    }

    Ok(())
}
