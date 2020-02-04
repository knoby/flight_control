use gio::prelude::*;
use gtk::prelude::*;

use gilrs::{Axis, Button, EventType, Gilrs};

use serial::prelude::*;

use std::io::Write;

use std::env::args;

fn build_ui(application: &gtk::Application) {
    // Gamepad Input
    let mut gilrs = Gilrs::new().expect("Failed to create gilrs context");

    // Wait for Select to be pressd. Thiss will be the active gamepad
    let mut gamepad = None;
    while gamepad.is_none() {
        while let Some(ev) = gilrs.next_event() {
            if let EventType::ButtonPressed(Button::Select, ..) = ev.event {
                gamepad = Some(ev.id);
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!(
        "Gamepad {} selected",
        gilrs.gamepad(gamepad.unwrap()).name()
    );

    // Load ui definition from file
    let builder = gtk::Builder::new_from_file("gtk_ui/flight_control.glade");

    // Get the window from the file
    let main_win: gtk::ApplicationWindow = builder.get_object("main_window").unwrap();

    main_win.set_application(Some(application));

    // get button for handling test
    let button_a: gtk::CheckButton = builder.get_object("button_state_a").unwrap();
    let button_b: gtk::CheckButton = builder.get_object("button_state_b").unwrap();
    let button_c: gtk::CheckButton = builder.get_object("button_state_c").unwrap();
    let button_x: gtk::CheckButton = builder.get_object("button_state_x").unwrap();
    let button_y: gtk::CheckButton = builder.get_object("button_state_y").unwrap();
    let button_z: gtk::CheckButton = builder.get_object("button_state_z").unwrap();
    let button_select: gtk::CheckButton = builder.get_object("button_state_select").unwrap();
    let button_rt: gtk::CheckButton = builder.get_object("button_state_rt").unwrap();
    let button_lt: gtk::CheckButton = builder.get_object("button_state_lt").unwrap();
    let stick_left_v: gtk::ProgressBar = builder.get_object("stick_left_state_h").unwrap();
    let stick_left_h: gtk::ProgressBar = builder.get_object("stick_left_state_v").unwrap();
    let stick_right_v: gtk::ProgressBar = builder.get_object("stick_right_state_h").unwrap();
    let stick_right_h: gtk::ProgressBar = builder.get_object("stick_right_state_v").unwrap();
    let stick_throttle: gtk::ProgressBar = builder.get_object("stick_throttle_state").unwrap();
    let button_u: gtk::CheckButton = builder.get_object("button_state_u").unwrap();
    let button_d: gtk::CheckButton = builder.get_object("button_state_d").unwrap();
    let button_l: gtk::CheckButton = builder.get_object("button_state_l").unwrap();
    let button_r: gtk::CheckButton = builder.get_object("button_state_r").unwrap();

    // Create Serial Connection
    let mut port = serial::open("/dev/ttyACM0").unwrap();
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud9600).unwrap();
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })
    .unwrap();

    port.set_timeout(std::time::Duration::from_millis(1000))
        .unwrap();

    timeout_add(20, move || {
        // Show current Input State to GUI
        while let Some(env) = gilrs.next_event() {
            use EventType::*;
            match env.event {
                AxisChanged(Axis::Unknown, value, ..) => {
                    stick_throttle.set_fraction(1.0 * (-value as f64 + 1.0) / 2.0);
                }
                ButtonPressed(Button::LeftTrigger, _) => {
                    let mut msg = heapless::Vec::<u8, heapless::consts::U32>::new();
                    copter_defs::Command::ToggleLed
                        .to_slip(&mut msg)
                        .and_then(|_| {
                            port.write_all(msg.as_ref()).unwrap();
                            Ok(())
                        })
                        .unwrap();
                }
                ButtonPressed(Button::RightTrigger, _) => {
                    let mut msg = heapless::Vec::<u8, heapless::consts::U32>::new();
                    copter_defs::Command::GetMotionState
                        .to_slip(&mut msg)
                        .and_then(|_| {
                            port.write_all(msg.as_ref()).unwrap();
                            Ok(())
                        })
                        .expect("Unable to decode GetMotionStateCommand");
                    use std::io::Read;
                    let mut msg_buffer = [0; 64];
                    msg.clear();
                    while let Ok(len) = port.read(&mut msg_buffer) {
                        // Read Until Error occured
                        for byte in msg_buffer[..len].iter() {
                            if *byte == rc_framing::framing::END {
                                if let Ok(cmd) = copter_defs::Command::from_slip(&msg) {
                                    println!("{:#?}", cmd);
                                }
                            } else {
                                msg.push(*byte).unwrap();
                            };
                        }
                    }
                }
                _ => (),
            };
        }
        button_a.set_active(gilrs.gamepad(gamepad.unwrap()).is_pressed(Button::South));
        button_b.set_active(gilrs.gamepad(gamepad.unwrap()).is_pressed(Button::East));
        button_c.set_active(
            gilrs
                .gamepad(gamepad.unwrap())
                .is_pressed(Button::RightTrigger2),
        );
        button_x.set_active(gilrs.gamepad(gamepad.unwrap()).is_pressed(Button::West));
        button_y.set_active(gilrs.gamepad(gamepad.unwrap()).is_pressed(Button::North));
        button_z.set_active(
            gilrs
                .gamepad(gamepad.unwrap())
                .is_pressed(Button::LeftTrigger2),
        );
        button_select.set_active(gilrs.gamepad(gamepad.unwrap()).is_pressed(Button::Select));
        button_lt.set_active(
            gilrs
                .gamepad(gamepad.unwrap())
                .is_pressed(Button::LeftTrigger),
        );
        button_rt.set_active(
            gilrs
                .gamepad(gamepad.unwrap())
                .is_pressed(Button::RightTrigger),
        );
        button_u.set_active(gilrs.gamepad(gamepad.unwrap()).is_pressed(Button::DPadUp));
        button_d.set_active(gilrs.gamepad(gamepad.unwrap()).is_pressed(Button::DPadDown));
        button_l.set_active(gilrs.gamepad(gamepad.unwrap()).is_pressed(Button::DPadLeft));
        button_r.set_active(
            gilrs
                .gamepad(gamepad.unwrap())
                .is_pressed(Button::DPadRight),
        );
        if let Some(axis_data) = gilrs.gamepad(gamepad.unwrap()).axis_data(Axis::LeftStickX) {
            stick_left_v.set_fraction((axis_data.value() as f64 + 1.0) / 2.0);
        }
        if let Some(axis_data) = gilrs.gamepad(gamepad.unwrap()).axis_data(Axis::LeftStickY) {
            stick_left_h.set_fraction((axis_data.value() as f64 + 1.0) / 2.0);
        }
        if let Some(axis_data) = gilrs.gamepad(gamepad.unwrap()).axis_data(Axis::RightStickX) {
            stick_right_v.set_fraction((axis_data.value() as f64 + 1.0) / 2.0);
        }
        if let Some(axis_data) = gilrs.gamepad(gamepad.unwrap()).axis_data(Axis::RightStickY) {
            stick_right_h.set_fraction((axis_data.value() as f64 + 1.0) / 2.0);
        }
        Continue(true)
    });

    main_win.show_all();
}

fn main() {
    // GTK
    let fc_app = gtk::Application::new(None, Default::default()).expect("Application::new failed");

    fc_app.connect_activate(|app| {
        build_ui(app);
    });

    fc_app.run(&args().collect::<Vec<_>>());
}
