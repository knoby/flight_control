use gilrs::{Axis, Button, EventType, Gilrs};

use serial::prelude::*;

use std::io::Write;

use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph, Text},
    Terminal,
};

// Local Modules
mod util;
use util::event::{Event, Events};

struct App {
    update_counter: u32,
    rx_counter: u32,
    tx_counter: u32,
    serial_port: Box<dyn SerialPort>,
}

impl App {
    /// Create a new Instance of the application
    fn new(serial_port: Box<dyn SerialPort>) -> Self {
        App {
            update_counter: 0,
            rx_counter: 0,
            tx_counter: 0,
            serial_port,
        }
    }

    /// Update the Input State and and read/send data to serial
    fn update(&mut self) {
        // Track number of updates
        self.update_counter += 1;
    }

    /// Handle Key Input
    fn handle_key_input(&mut self, key: Key) {
        match key {
            Key::Char('t') => {
                // Recived Toggle LED Key for debuging things
                let send_data = copter_defs::Command::ToggleLed.to_slip();
                self.serial_port.as_mut().write(&send_data);
            }
            _ => (),
        }
    }

    /// Draw the status of the app to the screen
    fn draw<T>(&mut self, terminal: &mut Terminal<TermionBackend<T>>) -> Result<(), std::io::Error>
    where
        T: Write,
    {
        terminal.draw(|mut f| {
            let size = f.size();
            let text = [
                Text::raw(format!("Update Counter: {}\n", self.update_counter)),
                Text::raw(format!("TX Counter: {}\n", self.tx_counter)),
                Text::raw(format!("RX Counter: {}\n", self.rx_counter)),
            ];
            let par = Paragraph::new(text.iter());
            f.render_widget(par, size)
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
        .map_err(|_| format!("Unable to init serial communication"))?;

    // Create new Window
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    let mut app = App::new(Box::new(port));

    loop {
        match events.next()? {
            Event::Input(Key::Char('q')) => {
                break;
            }
            Event::Input(key) => app.handle_key_input(key),
            Event::Tick => {
                app.update();
            }
        }

        app.draw(&mut terminal)?;
    }

    Ok(())
}
