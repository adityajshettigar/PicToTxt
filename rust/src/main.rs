#![forbid(unsafe_code)]

mod app;
mod drop_area;
mod image_input;
mod ocr;
mod util;
mod window;

use adw::prelude::*;

fn main() -> gtk::glib::ExitCode {
    // Default flags: a second invocation activates the already-running
    // instance over D-Bus, which raises its window; the new process exits.
    let app = adw::Application::builder()
        .application_id(app::APP_ID)
        .build();
    app.connect_startup(|app| {
        app::load_css();
        app::setup_actions(app);
    });
    app.connect_activate(window::present_or_create);
    app.run()
}
