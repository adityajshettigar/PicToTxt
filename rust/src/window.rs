use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gdk, gio, glib};

use crate::image_input::{self, Pasted};
use crate::ocr::{self, Job, OcrResult};
use crate::{app, drop_area, util};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Idle,
    Working,
    Ready,
    Error(String),
}

pub struct AppState {
    pub window: adw::ApplicationWindow,
    toast_overlay: adw::ToastOverlay,
    spinner: gtk::Spinner,
    status: gtk::Label,
    text_view: gtk::TextView,
    clear_btn: gtk::Button,
    copy_btn: gtk::Button,
    state: RefCell<State>,
    settings: Option<gio::Settings>,
}

impl AppState {
    pub fn toast(&self, msg: &str) {
        self.toast_overlay.add_toast(adw::Toast::new(msg));
    }

    fn set_state(&self, new: State) {
        match &new {
            State::Idle => {
                self.spinner.stop();
                self.spinner.set_visible(false);
                self.status.set_text("");
                self.text_view.buffer().set_text("");
                self.copy_btn.set_sensitive(false);
                self.clear_btn.set_sensitive(false);
            }
            State::Working => {
                self.spinner.set_visible(true);
                self.spinner.start();
                self.status.set_text("Extracting text…");
                self.copy_btn.set_sensitive(false);
                self.clear_btn.set_sensitive(false);
            }
            State::Ready => {
                self.spinner.stop();
                self.spinner.set_visible(false);
                self.status.set_text("Done");
                self.copy_btn.set_sensitive(true);
                self.clear_btn.set_sensitive(true);
            }
            State::Error(msg) => {
                self.spinner.stop();
                self.spinner.set_visible(false);
                self.status.set_text(msg);
                self.copy_btn.set_sensitive(false);
                self.clear_btn.set_sensitive(true);
            }
        }
        *self.state.borrow_mut() = new;
    }

    fn is_state(&self, wanted: &State) -> bool {
        *self.state.borrow() == *wanted
    }

    fn auto_copy_enabled(&self) -> bool {
        self.settings
            .as_ref()
            .is_some_and(|s| s.boolean("auto-copy"))
    }

    fn begin(self: &Rc<Self>, job: Job) {
        if self.is_state(&State::Working) {
            self.toast("Still working…");
            return;
        }
        self.set_state(State::Working);

        let rx = ocr::spawn(job);
        let this = self.clone();
        glib::MainContext::default().spawn_local(async move {
            match rx.recv().await {
                Ok(result) => this.finish(result),
                Err(_) => {
                    let msg = "OCR worker stopped".to_owned();
                    this.set_state(State::Error(msg.clone()));
                    this.toast(&msg);
                }
            }
        });
    }

    fn finish(&self, result: OcrResult) {
        match result {
            OcrResult::Ok { text } => {
                self.text_view.buffer().set_text(&text);
                self.set_state(State::Ready);
                if self.auto_copy_enabled() {
                    self.window.clipboard().set_text(&text);
                }
                self.toast("Text extracted");
            }
            OcrResult::Err(msg) => {
                self.set_state(State::Error(msg.clone()));
                self.toast(&msg);
            }
        }
    }

    pub fn process_path(self: &Rc<Self>, path: &Path) {
        if !util::is_valid_image_ext(path) {
            self.toast("Not an image");
            return;
        }
        self.begin(Job::Path(path.to_path_buf()));
    }

    pub fn process_texture(self: &Rc<Self>, texture: &gdk::Texture) {
        self.begin(Job::Bytes(image_input::texture_to_png(texture)));
    }

    fn paste(self: &Rc<Self>) {
        let this = self.clone();
        let clipboard = self.window.clipboard();
        glib::MainContext::default().spawn_local(async move {
            match image_input::read_clipboard(&clipboard).await {
                Pasted::Path(path) => this.process_path(&path),
                Pasted::Bytes(bytes) => this.begin(Job::Bytes(bytes)),
                Pasted::Nothing => this.toast("Nothing to paste"),
            }
        });
    }

    fn open_dialog(self: &Rc<Self>) {
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Images"));
        for mime in util::IMAGE_MIME_TYPES {
            filter.add_mime_type(mime);
        }
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let dialog = gtk::FileDialog::builder()
            .title("Choose an image")
            .modal(true)
            .filters(&filters)
            .default_filter(&filter)
            .build();

        let this = self.clone();
        dialog.open(Some(&self.window), gio::Cancellable::NONE, move |res| {
            // An Err here means the dialog was dismissed.
            if let Ok(file) = res {
                match file.path() {
                    Some(path) => this.process_path(&path),
                    None => this.toast("Not an image"),
                }
            }
        });
    }

    /// Copy the text. `selection_ok` lets a text selection win over the full text.
    fn copy(&self, selection_ok: bool) {
        if !self.is_state(&State::Ready) {
            self.toast("Nothing to copy");
            return;
        }
        let buffer = self.text_view.buffer();
        let text = match buffer.selection_bounds() {
            Some((start, end)) if selection_ok => buffer.text(&start, &end, false),
            _ => buffer.text(&buffer.start_iter(), &buffer.end_iter(), false),
        };
        self.window.clipboard().set_text(&text);
        self.toast("Copied");
    }

    fn clear(&self) {
        if self.is_state(&State::Ready) || matches!(*self.state.borrow(), State::Error(_)) {
            self.set_state(State::Idle);
        }
    }
}

fn add_window_action(window: &adw::ApplicationWindow, name: &str, f: impl Fn() + 'static) {
    let action = gio::SimpleAction::new(name, None);
    action.connect_activate(move |_, _| f());
    window.add_action(&action);
}

/// Present the existing window, or create a new one.
pub fn present_or_create(app: &adw::Application) {
    let existing = app
        .windows()
        .into_iter()
        .find_map(|w| w.downcast::<adw::ApplicationWindow>().ok());
    match existing {
        Some(window) => window.present(),
        None => build(app),
    }
}

fn build(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("PictoTxt")
        .default_width(720)
        .default_height(520)
        .width_request(480)
        .height_request(360)
        .build();

    // Header bar.
    let header = adw::HeaderBar::new();
    let title = gtk::Label::new(Some("PictoTxt"));
    title.add_css_class("title");
    header.set_title_widget(Some(&title));

    let menu = gio::Menu::new();
    menu.append(Some("Preferences"), Some("app.preferences"));
    menu.append(Some("About PictoTxt"), Some("app.about"));
    menu.append(Some("Quit"), Some("app.quit"));
    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .build();
    header.pack_end(&menu_button);

    // Content.
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let spinner = gtk::Spinner::new();
    spinner.set_visible(false);
    let status = gtk::Label::builder().xalign(0.0).build();
    let status_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    status_row.append(&spinner);
    status_row.append(&status);

    let text_view = gtk::TextView::builder()
        .editable(false)
        .cursor_visible(false)
        .wrap_mode(gtk::WrapMode::Word)
        .top_margin(8)
        .bottom_margin(8)
        .left_margin(8)
        .right_margin(8)
        .build();
    let scrolled = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .has_frame(true)
        .child(&text_view)
        .build();

    let clear_btn = gtk::Button::with_label("Clear");
    clear_btn.set_sensitive(false);
    let copy_btn = gtk::Button::with_label("Copy");
    copy_btn.add_css_class("suggested-action");
    copy_btn.set_sensitive(false);
    copy_btn.set_tooltip_text(Some(&format!(
        "Copy ({})",
        util::format_accelerator("<Primary>c")
    )));
    let button_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::End)
        .build();
    button_row.append(&clear_btn);
    button_row.append(&copy_btn);

    content.append(&status_row);
    content.append(&scrolled);
    content.append(&button_row);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&content));

    let toast_overlay = adw::ToastOverlay::new();
    toast_overlay.set_child(Some(&toolbar));
    window.set_content(Some(&toast_overlay));

    // Settings (optional: only present once the extension is installed).
    let settings = app::load_settings();
    if let Some(settings) = &settings {
        text_view.set_monospace(settings.boolean("monospace"));
        let tv = text_view.clone();
        settings.connect_changed(Some("monospace"), move |s, _| {
            tv.set_monospace(s.boolean("monospace"));
        });
    }

    let state = Rc::new(AppState {
        window: window.clone(),
        toast_overlay,
        spinner,
        status,
        text_view,
        clear_btn,
        copy_btn,
        state: RefCell::new(State::Idle),
        settings,
    });

    // Drop zone goes first in the content box.
    content.prepend(&drop_area::build(state.clone()));

    // Buttons.
    {
        let st = state.clone();
        state.copy_btn.connect_clicked(move |_| st.copy(false));
    }
    {
        let st = state.clone();
        state.clear_btn.connect_clicked(move |_| st.clear());
    }

    // Window actions (accelerators are set in app::setup_actions).
    {
        let st = state.clone();
        add_window_action(&window, "paste", move || st.paste());
    }
    {
        let st = state.clone();
        add_window_action(&window, "open", move || st.open_dialog());
    }
    {
        let st = state.clone();
        add_window_action(&window, "copy", move || st.copy(true));
    }
    {
        let weak = window.downgrade();
        add_window_action(&window, "close", move || {
            if let Some(w) = weak.upgrade() {
                w.close();
            }
        });
    }

    window.present();
}
