mod block;
mod style;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::{
    prelude::*, ActionBar, EventControllerFocus, MessageType, ScrolledWindow,
    TextBuffer, TextMark,
};
use relm4::{send, ComponentUpdate, Components, Widgets};
use rustybrain_core::kasten::Kasten;
use rustybrain_core::md::TreeCursor;
use rustybrain_core::md::{Node, Tree};
use rustybrain_core::zettel::Zettel;

use self::block::Blocking;

pub enum Msg {
    Open(Zettel),
    Insert(Zettel),
    OpenOnStack(Zettel),
    Changed,
    Save,
    Cursor,
    EditTitle,
    DoneEditTitle,
    InsertText(TextMark, String),
}

/// Zettel that be editing.
pub struct EditingZettel {
    title: gtk::EntryBuffer,
    buffer: gtk::TextBuffer,
    zettel: Zettel,
}

pub struct Model {
    tree: Option<Tree>,
    zettel: Option<Zettel>,
    kasten: Rc<RefCell<Kasten>>,

    stack: Vec<EditingZettel>,

    buffer: gtk::TextBuffer,
    title: gtk::EntryBuffer,
    view: gtk::TextView,

    editing_title: bool,

    blocks: Vec<block::Block>,
}

pub struct EditorComponents {}

impl Components<Model> for EditorComponents {
    fn init_components(
        _parent_model: &Model,
        _parent_sender: relm4::Sender<Msg>,
    ) -> Self {
        EditorComponents {}
    }

    fn connect_parent(&mut self, _parent_widgets: &Editor) {}
}

impl relm4::Model for Model {
    type Msg = Msg;

    type Widgets = Editor;

    type Components = EditorComponents;
}

pub struct Editor {
    layout: gtk::Box,
    main_win: gtk::ScrolledWindow,
    title_in: gtk::Entry,
    title_label: gtk::Label,
    title_show: gtk::Box,
    action_bar: gtk::ActionBar,
    save_btn: gtk::Button,
}

impl Model {
    fn buffer(&self) -> &TextBuffer {
        // check if stack is empty
        if let Some(b) = self.stack.last() {
            &b.buffer
        } else {
            &self.buffer
        }
    }

    fn zettel(&self) -> Option<Zettel> {
        // check if stack is empty
        if let Some(b) = self.stack.last() {
            Some(b.zettel.clone())
        } else {
            self.zettel.clone()
        }
    }

    fn title(&self) -> &gtk::EntryBuffer {
        if let Some(b) = self.stack.last() {
            &b.title
        } else {
            &self.title
        }
    }

    fn open_zettel(&mut self, z: Zettel) {
        self.buffer.set_text(z.content());
        self.title.set_text(z.title());
        self.zettel = Some(z);
    }

    fn open_zettel_on_stack(
        &mut self,
        zettel: Zettel,
        sender: relm4::Sender<Msg>,
    ) {
        let title = Self::create_title_buffer();
        let buffer = Self::create_buffer();
        buffer.set_text(zettel.content());
        title.set_text(zettel.title());

        let ez = EditingZettel {
            title,
            buffer,
            zettel,
        };
        self.stack.push(ez);
        self.listen_buffer_event(sender);
    }

    fn pop_stack_and_insert(&mut self, sender: relm4::Sender<Msg>) {
        if let Some(z) = self.stack.pop() {
            send!(sender, Msg::Insert(z.zettel));
        }
    }

    fn listen_buffer_event(&self, sender: relm4::Sender<Msg>) {
        let s = sender.clone();

        self.buffer()
            .connect_changed(move |_| send!(s, Msg::Changed));

        let s = sender.clone();
        self.buffer().connect_insert_text(move |b, i, t| {
            send!(
                s,
                Msg::InsertText(b.create_mark(None, i, false), t.to_string())
            )
        });

        let s = sender.clone();
        self.buffer()
            .connect_cursor_position_notify(move |_| send!(s, Msg::Cursor));
    }

    fn create_buffer() -> TextBuffer {
        let table = style::Style::new().table();
        gtk::TextBuffer::builder()
            .enable_undo(true)
            .tag_table(&table)
            .build()
    }

    fn create_title_buffer() -> gtk::EntryBuffer {
        gtk::EntryBuffer::builder().build()
    }

    fn on_buffer_changed(&mut self) {
        while let Some(blk) = self.blocks.pop() {
            blk.umount(&self.view, self.buffer());
        }

        let start = self.buffer().start_iter();
        let end = self.buffer().end_iter();

        self.buffer().remove_all_tags(&start, &end);
        self.buffer().apply_tag_by_name("p", &start, &end);

        let text = self.buffer().text(&start, &end, true);

        if let Ok(tree) = rustybrain_core::md::parse(text.as_str(), None) {
            self.tree = tree;
        } else {
            self.tree = None;
        }

        if let Some(tree) = self.tree.clone() {
            self.walk(tree.walk());
        }
    }

    fn walk(&mut self, mut cursor: TreeCursor) {
        let mut nodes_to_deep = vec![cursor.node()];
        while let Some(node) = nodes_to_deep.pop() {
            self.on_node(&node);
            cursor.reset(node);
            if cursor.goto_first_child() {
                nodes_to_deep.push(cursor.node());
                while cursor.goto_next_sibling() {
                    nodes_to_deep.push(cursor.node());
                }
            }
        }
    }

    fn on_node(&mut self, node: &Node) {
        let blk = block::Block::from_node(node, self.buffer());
        blk.mount(&self.view, self.buffer());
        self.blocks.push(blk);
    }

    fn on_cursor_notify(&mut self) {
        let offset = self.buffer().cursor_position();

        for blk in &self.blocks {
            if blk.is_anonymous() {
                continue;
            }

            if blk.start(self.buffer()).offset() <= offset
                && blk.end(self.buffer()).offset() > offset
            {
                blk.cursor_in(&self.view, self.buffer())
            } else {
                blk.cursor_out(&self.view, self.buffer())
            }
        }
    }
}

impl ComponentUpdate<super::AppModel> for Model {
    fn init_model(parent_model: &super::AppModel) -> Self {
        let buffer = Self::create_buffer();
        let title = Self::create_title_buffer();

        let view = gtk::TextView::builder()
            .buffer(&buffer)
            .vexpand(true)
            .hexpand(true)
            .pixels_inside_wrap(10)
            .wrap_mode(gtk::WrapMode::Char)
            .build();

        let mut model = Model {
            tree: None,
            zettel: None,
            stack: vec![],
            kasten: parent_model.kasten.clone(),
            blocks: vec![],
            editing_title: false,
            buffer,
            title,
            view,
        };
        model.on_buffer_changed();
        model
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        sender: relm4::Sender<Self::Msg>,
        parent_sender: relm4::Sender<super::Msg>,
    ) {
        match msg {
            Msg::Changed => {
                self.buffer().set_modified(true);
                self.on_buffer_changed()
            }
            Msg::Cursor => self.on_cursor_notify(),
            Msg::Open(z) => {
                self.editing_title = false;
                self.open_zettel(z)
            }

            Msg::OpenOnStack(z) => {
                self.editing_title = false;
                self.open_zettel_on_stack(z, sender);
            }
            Msg::Insert(z) => {
                self.buffer().insert_at_cursor(&format!(
                    "[{}]({})",
                    z.title(),
                    z.id(),
                ));
            }
            Msg::InsertText(m, s) => {
                if s == "@" {
                    let mut iter = self.buffer().iter_at_mark(&m);
                    let anchor = self.buffer().create_child_anchor(&mut iter);
                    let child =
                        gtk::Label::builder().label("You Complete Me").build();
                    self.view.add_child_at_anchor(&child, &anchor);
                }
            }
            Msg::Save => {
                if self.buffer().is_modified() {
                    let start = self.buffer().start_iter();
                    let end = self.buffer().end_iter();
                    let text = self.buffer().text(&start, &end, true);
                    let title = self.title().text();
                    if let Some(zettel) = self.zettel().as_mut() {
                        zettel.set_content(&text);
                        zettel.set_title(&title);
                        let mut kasten = self.kasten.borrow_mut();
                        if let Err(err) = kasten.save(zettel) {
                            send!(
                                parent_sender,
                                super::Msg::ShowMsg(
                                    MessageType::Error,
                                    format!("Save note failed: {:?}", err)
                                )
                            );
                        }
                    }
                    self.buffer().set_modified(false);
                    self.pop_stack_and_insert(sender);
                }
            }
            Msg::EditTitle => self.editing_title = true,
            Msg::DoneEditTitle => {
                self.editing_title = false;
            }
        }
    }
}

impl Widgets<Model, super::AppModel> for Editor {
    type Root = gtk::Box;

    fn root_widget(&self) -> Self::Root {
        self.layout.clone()
    }

    fn init_view(
        model: &Model,
        _components: &EditorComponents,
        sender: relm4::Sender<Msg>,
    ) -> Self {
        let box_ = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .margin_end(10)
            .build();

        let label = gtk::Label::builder().hexpand(true).vexpand(false).build();

        let entry = gtk::Entry::builder()
            .hexpand(true)
            .vexpand(false)
            .placeholder_text("Title")
            .buffer(model.title())
            .build();

        let focus_ctrl = EventControllerFocus::builder().build();
        let s = sender.clone();
        focus_ctrl.connect_leave(move |_| send!(s, Msg::DoneEditTitle));
        entry.add_controller(&focus_ctrl);

        let window = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .child(&model.view)
            .build();
        model.listen_buffer_event(sender.clone());

        let title_show = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .hexpand(true)
            .vexpand(false)
            .build();
        let edit_btn = gtk::Button::builder().label("Edit").build();
        let s = sender.clone();
        edit_btn.connect_clicked(move |_| send!(s, Msg::EditTitle));
        title_show.append(&label);
        title_show.append(&edit_btn);

        let action_bar = ActionBar::builder().build();
        let save_btn = gtk::Button::builder().label("Save").build();
        let s = sender.clone();
        save_btn.connect_clicked(move |_| send!(s, Msg::Save));
        action_bar.pack_end(&save_btn);

        Editor {
            layout: box_,
            title_in: entry,
            title_label: label,
            title_show,
            main_win: window,
            action_bar,
            save_btn,
        }
    }

    fn view(&mut self, model: &Model, _sender: relm4::Sender<Msg>) {
        loop {
            match self.layout.last_child() {
                Some(c) => self.layout.remove(&c),
                None => break,
            }
        }
        model.view.set_buffer(Some(model.buffer()));
        self.title_in.set_buffer(model.title());
        if model.editing_title {
            self.layout.append(&self.title_in);
        } else {
            self.layout.append(&self.title_show);
        }
        if model.buffer.is_modified() {
            self.save_btn.set_sensitive(true);
        } else {
            self.save_btn.set_sensitive(false);
        }
        self.layout.append(&self.action_bar);
        self.layout.append(&self.main_win);
        model.view.grab_focus();
        self.title_label.set_text(&model.title().text());
        if model.title().text() == "" {
            self.title_in.set_placeholder_text(Some("Title"))
        } else {
            self.title_in.set_placeholder_text(None)
        }
    }
}
