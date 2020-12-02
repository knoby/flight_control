// Things from relm
use relm::{connect, Relm};
use relm_derive::Msg;

// GTK Imports
use gtk::prelude::*;

pub struct Model {}

#[derive(Msg)]
pub enum Message {
    EnableMotor,
    DisableMotor,
    SendSetPoint(copter_com::SetValues),
}

pub struct Widget {
    _model: Model,
    root: gtk::Frame,
}

impl relm::Update for Widget {
    type Model = Model;
    type ModelParam = ();
    type Msg = Message;

    fn model(_relm: &Relm<Self>, _param: Self::ModelParam) -> Self::Model {
        Model {}
    }

    fn update(&mut self, _event: Self::Msg) {}
}

impl relm::Widget for Widget {
    type Root = gtk::Frame;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(relm: &Relm<Self>, _model: Self::Model) -> Self {
        let root = gtk::Frame::new(Some("Control"));
        let root_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        root.add(&root_box);

        // Buttons for enable disable motors
        let box_motors = gtk::ButtonBox::new(gtk::Orientation::Vertical);
        root_box.add(&box_motors);
        let btn_enable_motor = gtk::Button::new();
        btn_enable_motor.set_label("Enable Motor");
        box_motors.add(&btn_enable_motor);
        let btn_disable_motor = gtk::Button::new();
        btn_disable_motor.set_label("Disable Motor");
        box_motors.add(&btn_disable_motor);

        // Buttons for control Mode
        let box_mode = gtk::ButtonBox::new(gtk::Orientation::Vertical);
        root_box.add(&box_mode);

        let btn_sequence = gtk::Button::new();
        btn_sequence.set_label("Sequence Test");
        box_mode.add(&btn_sequence);

        let btn_direct_ctrl = gtk::Button::new();
        btn_direct_ctrl.set_label("Direct Control");
        box_mode.add(&btn_direct_ctrl);

        let btn_pryt_ctrl = gtk::Button::new();
        btn_pryt_ctrl.set_label("PRYT Control");
        box_mode.add(&btn_pryt_ctrl);

        let btn_stabalize = gtk::Button::new();
        btn_stabalize.set_label("Stabalize");
        box_mode.add(&btn_stabalize);

        let btn_angle_ctrl = gtk::Button::new();
        btn_angle_ctrl.set_label("Angle Control");
        box_mode.add(&btn_angle_ctrl);

        // Connect Button events
        connect!(
            relm,
            btn_enable_motor,
            connect_clicked(_),
            Message::EnableMotor
        );
        connect!(
            relm,
            btn_disable_motor,
            connect_clicked(_),
            Message::DisableMotor
        );
        connect!(
            relm,
            btn_sequence,
            connect_clicked(_),
            Message::SendSetPoint(copter_com::SetValues::SequenceTest)
        );
        connect!(
            relm,
            btn_direct_ctrl,
            connect_clicked(_),
            Message::SendSetPoint(copter_com::SetValues::DirectControl((
                10.0, 10.0, 10.0, 10.0
            )))
        );

        Self { root, _model }
    }
}
