// Things from relm
use relm::{connect, Relm};
use relm_derive::Msg;

// GTK Imports
use gtk::prelude::*;
use gtk::ComboBoxTextExt;
use gtk::{Button, Frame};

// Serial Imports
use serialport::prelude::*;

pub struct Model {
    root: Frame,
    device_list: gtk::ComboBoxText,
    btn_connect: gtk::Button,
    btn_disconnect: gtk::Button,
    btn_refresh: gtk::Button,
    serial: Option<Box<dyn SerialPort>>,
    app_reciver: Option<relm::Channel<Message>>,
    app_sender: Option<std::sync::mpsc::Sender<Vec<u8>>>,
    relm: relm::Relm<Widget>,
    led: bool,
}

#[derive(Msg)]
pub enum Message {
    Connect,
    Disconnect,
    RefreshDeviceList,
    ConnectionError,
    KeepAlive,
}

pub struct Widget {
    model: Model,
}

impl Widget {
    fn refresh_device_list(&self) {
        if let Ok(device_list) = serialport::available_ports() {
            self.model.device_list.remove_all();
            for device in device_list.iter() {
                self.model
                    .device_list
                    .append(Some(&device.port_name), &device.port_name);
            }
            if !device_list.is_empty() {
                self.model
                    .device_list
                    .set_active_id(Some(&device_list[0].port_name));
            }
        };
    }

    fn disconnect(&mut self) {
        self.model.serial.take();
        self.model.app_reciver.take();
        self.model.app_sender.take();
    }

    fn connect(&mut self) {
        let port_settings = serialport::SerialPortSettings {
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: std::time::Duration::from_millis(50),
        };
        // ====
        // Open a connection. If successfull spawn a thread to handle the connection.
        // The thread sends a message to indicate a failure of the connection.
        // The thread observes the channel to end the thread if the channel is droped
        // The Application can send a message to the thread to close the connection
        // ====
        let port = self
            .model
            .device_list
            .get_active_text()
            .unwrap_or_else(|| "".into());
        if let Ok(mut serial) = serialport::open_with_settings(&port, &port_settings) {
            // Create the channels from the thread and to the thread
            let stream = self.model.relm.stream().clone();
            let (app_reciver, thread_sender) =
                relm::Channel::<Message>::new(move |msg| stream.emit(msg));
            let (app_sender, thread_reciver) = std::sync::mpsc::channel::<Vec<u8>>();

            std::thread::spawn(move || {
                let timeout = std::time::Duration::from_millis(50);
                loop {
                    // ====
                    // check for new message to send
                    match thread_reciver.recv_timeout(timeout) {
                        Ok(buffer) => {
                            // try to send the data
                            if serial.write_all(buffer.as_ref()).is_err() {
                                thread_sender.send(Message::ConnectionError).ok(); // we don't handle the error because the thread ends here
                                break; // on error drop connection
                            }
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                            break;
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (), // repeat the loop
                    }
                }
            });

            // save the sender/reciver in the model
            self.model.app_reciver = Some(app_reciver);
            self.model.app_sender = Some(app_sender);
        } else {
            self.model.relm.stream().emit(Message::ConnectionError);
        }
    }

    fn enable_connect(&self) {
        self.model.btn_refresh.set_sensitive(true);
        self.model.btn_connect.set_sensitive(true);
        self.model.btn_disconnect.set_sensitive(false);
        self.model.device_list.set_sensitive(true);
    }

    fn disable_connect(&self) {
        self.model.btn_refresh.set_sensitive(false);
        self.model.btn_connect.set_sensitive(false);
        self.model.btn_disconnect.set_sensitive(true);
        self.model.device_list.set_sensitive(false);
    }
}

impl relm::Update for Widget {
    type Model = Model;
    type ModelParam = gtk::Builder;
    type Msg = Message;

    fn model(relm: &Relm<Self>, param: Self::ModelParam) -> self::Model {
        let root = param.get_object("FrameConnection").unwrap();

        // connect btn events
        let btn_connect: Button = param.get_object("BtnConnect").unwrap();
        connect!(relm, btn_connect, connect_clicked(_), Message::Connect);
        let btn_disconnect: Button = param.get_object("BtnDisconnect").unwrap();
        connect!(
            relm,
            btn_disconnect,
            connect_clicked(_),
            Message::Disconnect
        );
        let btn_refresh: Button = param.get_object("BtnRefresh").unwrap();
        connect!(
            relm,
            btn_refresh,
            connect_clicked(_),
            Message::RefreshDeviceList
        );

        // keepalive event
        relm::interval(relm.stream(), 500, || Message::KeepAlive);

        // Get the Device list combo box
        let device_list = param.get_object("ComboSerialDevice").unwrap();
        // Trigger filling of the devicelist
        relm.stream().emit(Message::RefreshDeviceList);

        Model {
            root,
            serial: None,
            app_reciver: None,
            app_sender: None,
            device_list,
            relm: relm.clone(),
            btn_connect,
            btn_disconnect,
            btn_refresh,
            led: false,
        }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Message::Connect => {
                self.disable_connect();
                self.connect();
            }
            Message::Disconnect => {
                self.disconnect();
                self.enable_connect();
            }
            Message::RefreshDeviceList => self.refresh_device_list(),
            Message::ConnectionError => {
                self.disconnect();
                self.enable_connect();
            }
            Message::KeepAlive => {
                if let Some(sender) = &mut self.model.app_sender {
                    let msg = if self.model.led {
                        b"SET Led=FALSE\n\r".to_vec()
                    } else {
                        b"SET Led=TRUE\n\r".to_vec()
                    };
                    sender.send(msg).ok();
                    self.model.led = !self.model.led;
                }
            }
        };
    }
}

impl relm::Widget for Widget {
    type Root = Frame;

    fn root(&self) -> Self::Root {
        self.model.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        Self { model }
    }
}
