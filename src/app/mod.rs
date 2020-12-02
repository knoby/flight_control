// Things from relm
use relm::{connect, Relm, Update, Widget};
use relm_derive::Msg;

// GTK Imports
use gtk::prelude::*;
use gtk::Window;
use relm::ContainerWidget;

use crate::widgets;

pub struct Model {}

#[derive(Msg)]
pub enum Message {
    Quit,
}

pub struct App {
    window: Window,
    _graph: relm::Component<widgets::graph::Widget>,
    _connection: relm::Component<widgets::connection::Widget>,
    _control: relm::Component<widgets::control::Widget>,
    _model: Model,
}

impl Update for App {
    type Model = Model;
    type Msg = Message;
    type ModelParam = ();

    fn model(_relm: &Relm<Self>, _param: Self::ModelParam) -> Self::Model {
        Model {}
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Message::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for App {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, _model: Self::Model) -> Self {
        let glade_src = include_str!("../../gtk_ui/main.glade");
        let builder = gtk::Builder::from_string(glade_src);

        let window: Window = builder.get_object("window").unwrap();
        let control_box: gtk::Box = builder.get_object("BoxControl").unwrap();
        let graph_box: gtk::Box = builder.get_object("BoxGraph").unwrap();

        let _connection = control_box.add_widget::<widgets::connection::Widget>(builder);
        let _graph = graph_box.add_widget::<widgets::graph::Widget>(());
        let _control = control_box.add_widget::<widgets::control::Widget>(());
        graph_box.set_child_expand(&graph_box.get_children()[0], true);

        window.show_all();

        // Close app
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Message::Quit), Inhibit(false))
        );

        // New data from device
        connect!(
            _connection@widgets::connection::Message::RecivedAttitude(ref data),
            _graph,
            widgets::graph::Message::AddAngle(data.timestamp, data.roll, data.pitch, data.yaw)
        );
        // Clear on new connect
        connect!(
            _connection@widgets::connection::Message::Connect,
            _graph,
            widgets::graph::Message::Clear
        );
        // Enable Motors
        connect!(
            _control@widgets::control::Message::EnableMotor,
            _connection,
            widgets::connection::Message::SendMessage(copter_com::Message::EnableMotor)
        );
        // Disable Motors
        connect!(
            _control@widgets::control::Message::DisableMotor,
            _connection,
            widgets::connection::Message::SendMessage(copter_com::Message::DisableMotor)
        );
        // Send Setpoint
        connect!(
            _control@widgets::control::Message::SendSetPoint(ref setpoint),
            _connection,
            widgets::connection::Message::SendMessage(copter_com::Message::ChangeSetvalue(*setpoint))
        );

        window.show_all();

        App {
            _model,
            window,
            _graph,
            _connection,
            _control,
        }
    }
}
