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
    _connection: relm::Component<widgets::connection::Widget>,
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

        let _connection = control_box.add_widget::<widgets::connection::Widget>(builder);

        window.show_all();

        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Message::Quit), Inhibit(false))
        );

        window.show_all();

        App {
            _model,
            window,
            _connection,
        }
    }
}
