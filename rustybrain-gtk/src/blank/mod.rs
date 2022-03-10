// # Nobody ever Starts From Scratch.
// This screen will lead you to steup a repository and bootstrap your second
// brain.
use gtk::{prelude::*, Frame};

pub enum Msg {}

pub struct Model {}

pub struct Blank {
    frame: gtk::Frame,
}

impl relm4::Model for Model {
    type Msg = Msg;

    type Widgets = Blank;

    type Components = ();
}

impl relm4::ComponentUpdate<super::AppModel> for Model {
    fn init_model(_parent_model: &super::AppModel) -> Self {
        Self {}
    }

    fn update(
        &mut self,
        _msg: Self::Msg,
        _components: &Self::Components,
        _sender: relm4::Sender<Self::Msg>,
        _parent_sender: relm4::Sender<super::Msg>,
    ) {
    }
}

impl relm4::Widgets<Model, super::AppModel> for Blank {
    type Root = gtk::Frame;

    fn init_view(
        _model: &Model,
        _components: &(),
        _sender: relm4::Sender<Msg>,
    ) -> Self {
        let assistant =
            gtk::Assistant::builder().modal(true).visible(true).build();
        let frame = Frame::builder()
            .label("Nobody ever Starts From Scratch")
            .child(&assistant)
            .vexpand(true)
            .hexpand(true)
            .build();
        Self { frame }
    }

    fn root_widget(&self) -> Self::Root {
        self.frame.clone()
    }

    fn view(&mut self, _model: &Model, _sender: relm4::Sender<Msg>) {}
}
