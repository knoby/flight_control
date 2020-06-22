use gilrs::Gilrs;

use serial::prelude::*;

use std::io::Write;

use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Chart, Gauge, Tabs};
use tui::{backend::TermionBackend, Terminal};

// Local Modules
mod util;
use util::event::{Event, Events};

struct App<'a> {
    update_counter: u32,
    tx_counter: u32,
    serial_port: Box<dyn SerialPort>,
    tab_state: util::TabsState<'a>,
    pitch: Vec<f32>,
    roll: Vec<f32>,
    motor: copter_defs::MotorState,
}

impl<'a> App<'a> {
    /// Create a new Instance of the application
    fn new(serial_port: Box<dyn SerialPort>) -> Self {
        // Create Interface
        let tab_state = util::TabsState::new(vec!["Main", "Connection", "Info"]);
        App {
            update_counter: 0,
            tx_counter: 0,
            serial_port,
            tab_state,
            pitch: Vec::new(),
            roll: Vec::new(),
            motor: copter_defs::MotorState::default(),
        }
    }

    /// Update the Input State and and read/send data to serial
    fn update(&mut self) {
        // Track number of updates
        self.update_counter += 1;
        if self.update_counter % 20 == 0 {
            self.send_op_code(copter_defs::Command::ToggleLed);
        }
        if self.update_counter % 5 == 0 {
            self.send_op_code(copter_defs::Command::GetMotionState);
            // Try to get the answer
            let mut msg = Vec::new();
            let mut buf = [0u8; 32];
            while let Ok(n) = self.serial_port.read(&mut buf) {
                buf.iter().take(n).for_each(|&byte| msg.push(byte));
            }
            if msg.len() >= 2 {
                msg.pop();
                msg.rotate_left(1);
                msg.pop();
            }
            if let Ok(copter_defs::Command::SendMotionState(state, armed)) =
                copter_defs::Command::from_slip(&msg)
            {
                self.roll
                    .push(state[0] / (2.0 * std::f32::consts::PI) * 360.0);
                if self.roll.len() > 20 {
                    self.roll.rotate_left(1);
                    self.roll.pop();
                }
                self.pitch
                    .push(state[1] / (2.0 * std::f32::consts::PI) * 360.0);
                if self.pitch.len() > 20 {
                    self.pitch.rotate_left(1);
                    self.pitch.pop();
                }
                self.motor.armed = armed;
            }
        }
    }

    /// Send Data to Serial and keep track of the send counter
    fn send_op_code(&mut self, op_code: copter_defs::Command) {
        // Inc TX Counter
        self.tx_counter += 1;
        // Encode Command
        let send_data = op_code.to_slip();
        self.serial_port.write_all(&send_data).unwrap();
    }

    /// Handle all keys from input chain
    fn handle_key(&mut self, key: termion::event::Key) {
        match key {
            Key::PageUp => self.tab_state.next(),
            Key::PageDown => self.tab_state.previous(),
            Key::Char('s') => {
                if self.motor.armed {
                    self.send_op_code(copter_defs::Command::StopMotor);
                } else {
                    self.send_op_code(copter_defs::Command::StartMotor);
                }
            }
            _ => (),
        }
    }

    /// Render the Connection tab
    fn render_connection_tab<T>(&mut self, mut f: tui::terminal::Frame<T>, area: tui::layout::Rect)
    where
        T: tui::backend::Backend,
    {
        use tui::widgets::{Paragraph, Text};
        let text = [Text::raw(format!("TX Count: {}", self.tx_counter))];
        f.render_widget(Paragraph::new(text.iter()), area);
    }

    /// Render the Main View with motorstate, graphs, control, ...
    fn render_main_tab<T>(&mut self, mut f: tui::terminal::Frame<T>, area: tui::layout::Rect)
    where
        T: tui::backend::Backend,
    {
        let layout_main = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);
        // Draw two boxes in the left and right area
        f.render_widget(Block::default().borders(Borders::ALL), layout_main[0]);
        // f.render_widget(Block::default().borders(Borders::ALL), layout_main[1]); // Is drawn by child elements

        // Create Layout for the display of Motor status and orientation
        let layout_status = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(layout_main[1]);
        // f.render_widget(Block::default().borders(Borders::ALL), layout_status[0]); // Is dran with the Chart
        f.render_widget(
            Block::default().borders(Borders::ALL).title("Motor Status"),
            layout_status[1],
        );

        // Area in which the motor state is drawn. on the left is some space for text. on the right will be four gauges
        let layout_motors = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(
                [
                    Constraint::Percentage(34),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(layout_status[1]);
        let layout_motor_state_left = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(layout_motors[1]);
        let layout_motor_state_right = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(layout_motors[2]);
        // Draw gauge with sample data
        let motor_fl = Gauge::default()
            .percent(self.motor.front_left as u16)
            .block(Block::default().borders(Borders::ALL).title("Front Left"))
            .style(Style::new().fg(if self.motor.armed {
                Color::LightGreen
            } else {
                Color::LightRed
            }));
        let motor_fr = motor_fl
            .clone()
            .percent(self.motor.front_right as u16)
            .block(Block::default().borders(Borders::ALL).title("Front Right"));
        let motor_rl = motor_fl
            .clone()
            .percent(self.motor.rear_left as u16)
            .block(Block::default().borders(Borders::ALL).title("Rear Left"));
        let motor_rr = motor_fl
            .clone()
            .percent(self.motor.rear_right as u16)
            .block(Block::default().borders(Borders::ALL).title("Rear Right"));
        f.render_widget(motor_fl.clone(), layout_motor_state_left[0]);
        f.render_widget(motor_fr.clone(), layout_motor_state_left[1]);
        f.render_widget(motor_rl.clone(), layout_motor_state_right[0]);
        f.render_widget(motor_rr.clone(), layout_motor_state_right[1]);

        // Area in which the Pitch and Roll anlge is plotted over time
        let data_roll: Vec<(f64, f64)> = self
            .roll
            .iter()
            .enumerate()
            .map(|(num, &angle)| (num as f64, angle as f64))
            .collect();
        let data_pitch: Vec<(f64, f64)> = self
            .pitch
            .iter()
            .enumerate()
            .map(|(num, &angle)| (num as f64, angle as f64))
            .collect();

        let roll = if self.roll.is_empty() {
            0.0
        } else {
            self.roll[self.roll.len() - 1]
        };
        let pitch = if self.pitch.is_empty() {
            0.0
        } else {
            self.pitch[self.pitch.len() - 1]
        };
        let dataset = [
            tui::widgets::Dataset::default()
                .name(format!("Roll {:>7.1}", roll))
                .marker(tui::symbols::Marker::Dot)
                .graph_type(tui::widgets::GraphType::Line)
                .style(Style::default().fg(Color::LightBlue))
                .data(&data_roll),
            tui::widgets::Dataset::default()
                .name(format!("Pitch {:>7.1}", pitch))
                .marker(tui::symbols::Marker::Dot)
                .graph_type(tui::widgets::GraphType::Line)
                .style(Style::default().fg(Color::LightGreen))
                .data(&data_pitch),
        ];
        let chart_rp = Chart::default()
            .block(Block::default().borders(Borders::ALL).title("Orientation"))
            .x_axis(
                tui::widgets::Axis::default()
                    .title("Time")
                    .bounds([0.0, self.roll.len() as f64])
                    .labels(&["", ""]),
            )
            .y_axis(
                tui::widgets::Axis::default()
                    .title("Angle °")
                    .bounds([-30.0, 30.0])
                    .labels(&["-30", "0", "30"]),
            )
            .datasets(&dataset);
        f.render_widget(chart_rp, layout_status[0]);
    }

    /// Draw the status of the app to the screen
    fn draw<T>(&mut self, terminal: &mut Terminal<T>) -> Result<(), std::io::Error>
    where
        T: tui::backend::Backend,
    {
        terminal.draw(|mut f| {
            // Devide view in Tab Zone and the Rest
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Length(3), Constraint::Percentage(100)].as_ref())
                .split(f.size());
            let tabs = Tabs::default()
                .block(Block::default().borders(Borders::ALL).title("Navigation"))
                .titles(&self.tab_state.titles)
                .select(self.tab_state.index)
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(Style::default().fg(Color::Yellow));
            let child_frame = Block::default().borders(Borders::ALL);
            // Render The Main things
            f.render_widget(tabs, main_layout[0]);
            f.render_widget(child_frame, main_layout[1]);
            // Render Childs
            match self.tab_state.index {
                0 => self.render_main_tab(f, child_frame.inner(main_layout[1])),
                1 => self.render_connection_tab(f, child_frame.inner(main_layout[1])),
                2 => (),
                _ => (),
            }
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Gamepad Input
    let _gilrs = Gilrs::new().map_err(|_| "Unable to init gamepad system".to_string())?;

    // Create Serial Connection
    let port = serial::open("/dev/ttyACM0")
        .and_then(|mut port| {
            port.reconfigure(&|settings| {
                settings.set_baud_rate(serial::Baud19200).unwrap();
                settings.set_char_size(serial::Bits8);
                settings.set_parity(serial::ParityNone);
                settings.set_stop_bits(serial::Stop1);
                settings.set_flow_control(serial::FlowNone);
                Ok(())
            })?;
            port.set_timeout(std::time::Duration::from_millis(5))?;
            Ok(port)
        })
        .map_err(|_| "Unable to init serial communication".to_string())?;

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
            Event::Input(key) => app.handle_key(key),
            Event::Tick => {
                app.update();
            }
        }

        app.draw(&mut terminal)?;
    }

    Ok(())
}
