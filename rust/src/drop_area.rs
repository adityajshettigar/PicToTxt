use std::path::Path;
use std::rc::Rc;

use adw::prelude::*;
use gtk::gdk;

use crate::image_input;
use crate::window::AppState;

/// Send a file path into the OCR pipeline.
pub fn process_path(state: &Rc<AppState>, path: &Path) {
    state.process_path(path);
}

/// Send a texture into the OCR pipeline.
pub fn process_texture(state: &Rc<AppState>, texture: &gdk::Texture) {
    state.process_texture(texture);
}

fn attach_hover(target: &gtk::DropTarget, overlay: &gtk::Overlay) {
    let on_enter = overlay.clone();
    target.connect_enter(move |_, _, _| {
        on_enter.add_css_class("hover");
        gdk::DragAction::COPY
    });
    let on_leave = overlay.clone();
    target.connect_leave(move |_| on_leave.remove_css_class("hover"));
}

/// Build the drop zone widget.
pub fn build(state: Rc<AppState>) -> gtk::Overlay {
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    let icon = gtk::Image::builder()
        .icon_name("image-x-generic-symbolic")
        .pixel_size(96)
        .opacity(0.7)
        .build();

    let title = gtk::Label::new(Some("Drop an image here"));
    title.add_css_class("title-3");

    let subtitle = gtk::Label::new(Some("or press Ctrl+V / Ctrl+O"));
    subtitle.add_css_class("dim-label");

    let choose = gtk::Button::with_label("Choose File…");
    choose.set_halign(gtk::Align::Center);
    choose.add_css_class("pill");
    choose.set_action_name(Some("win.open"));

    content.append(&icon);
    content.append(&title);
    content.append(&subtitle);
    content.append(&choose);

    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&content));
    overlay.add_css_class("drop-zone");

    // Drop target 1: file lists (e.g. from Nautilus).
    let files_target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
    attach_hover(&files_target, &overlay);
    {
        let state = state.clone();
        let overlay = overlay.clone();
        files_target.connect_drop(move |_, value, _, _| {
            overlay.remove_css_class("hover");
            let Ok(list) = value.get::<gdk::FileList>() else {
                state.toast("Not an image");
                return false;
            };
            let valid = image_input::image_paths(&list.files());
            match valid.first() {
                None => state.toast("Not an image"),
                Some(first) => {
                    if valid.len() > 1 {
                        state.toast("Only the first image was processed");
                    }
                    process_path(&state, first);
                }
            }
            true
        });
    }
    overlay.add_controller(files_target);

    // Drop target 2: raw textures (e.g. images dragged from a browser).
    let texture_target = gtk::DropTarget::new(gdk::Texture::static_type(), gdk::DragAction::COPY);
    attach_hover(&texture_target, &overlay);
    {
        let state = state.clone();
        let overlay = overlay.clone();
        texture_target.connect_drop(move |_, value, _, _| {
            overlay.remove_css_class("hover");
            match value.get::<gdk::Texture>() {
                Ok(texture) => {
                    process_texture(&state, &texture);
                    true
                }
                Err(_) => {
                    state.toast("Not an image");
                    false
                }
            }
        });
    }
    overlay.add_controller(texture_target);

    // Clicking anywhere on the zone opens the file chooser.
    let click = gtk::GestureClick::builder()
        .button(gdk::BUTTON_PRIMARY)
        .build();
    {
        let overlay = overlay.clone();
        click.connect_released(move |_, _, _, _| {
            let _ = overlay.activate_action("win.open", None);
        });
    }
    overlay.add_controller(click);

    overlay
}
