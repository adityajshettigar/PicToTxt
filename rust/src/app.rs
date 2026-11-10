use adw::prelude::*;
use gtk::{gdk, gio, glib};

pub const APP_ID: &str = "org.gnome.PictoTxt";
pub const SCHEMA_ID: &str = "org.gnome.shell.extensions.pictotxt";
const EXTENSION_UUID: &str = "pictotxt@example.github.io";

const CSS: &str = r#"
.drop-zone {
  border: 2px dashed alpha(currentColor, 0.35);
  border-radius: 12px;
  padding: 16px;
  min-height: 160px;
  transition: background 120ms ease, border-color 120ms ease;
}
.drop-zone.hover {
  border-color: @accent_bg_color;
  background: alpha(@accent_bg_color, 0.10);
}
"#;

/// Load the application stylesheet once at startup.
pub fn load_css() {
    let Some(display) = gdk::Display::default() else {
        eprintln!("picto-txt: no display, skipping CSS");
        return;
    };
    let provider = gtk::CssProvider::new();
    provider.load_from_data(CSS);
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

/// Open the extension's settings, if its schema is installed.
///
/// Looks in the system schema sources first, then in the extension's own
/// `schemas` directory. Returns `None` instead of panicking when missing.
pub fn load_settings() -> Option<gio::Settings> {
    let default_source = gio::SettingsSchemaSource::default();

    let mut schema = default_source
        .as_ref()
        .and_then(|source| source.lookup(SCHEMA_ID, true));

    if schema.is_none() {
        let dir = glib::user_data_dir()
            .join("gnome-shell")
            .join("extensions")
            .join(EXTENSION_UUID)
            .join("schemas");
        if dir.is_dir() {
            match gio::SettingsSchemaSource::from_directory(&dir, default_source.as_ref(), false) {
                Ok(source) => schema = source.lookup(SCHEMA_ID, false),
                Err(e) => eprintln!("picto-txt: cannot read schemas: {e}"),
            }
        }
    }

    let schema = schema?;
    Some(gio::Settings::new_full(
        &schema,
        None::<&gio::SettingsBackend>,
        None,
    ))
}

/// Register app-level actions and accelerators.
pub fn setup_actions(app: &adw::Application) {
    let preferences = gio::SimpleAction::new("preferences", None);
    let app_for_prefs = app.clone();
    preferences.connect_activate(move |_, _| show_preferences(&app_for_prefs));
    app.add_action(&preferences);

    let about = gio::SimpleAction::new("about", None);
    let app_for_about = app.clone();
    about.connect_activate(move |_, _| show_about(&app_for_about));
    app.add_action(&about);

    let quit = gio::SimpleAction::new("quit", None);
    let app_for_quit = app.clone();
    quit.connect_activate(move |_, _| app_for_quit.quit());
    app.add_action(&quit);

    app.set_accels_for_action("app.preferences", &["<Primary>comma"]);
    app.set_accels_for_action("win.paste", &["<Primary>v"]);
    app.set_accels_for_action("win.open", &["<Primary>o"]);
    app.set_accels_for_action("win.copy", &["<Primary>c"]);
    app.set_accels_for_action("win.close", &["Escape"]);
}

fn show_about(app: &adw::Application) {
    let about = adw::AboutWindow::builder()
        .application_name("PictoTxt")
        .application_icon("image-x-generic")
        .version(env!("CARGO_PKG_VERSION"))
        .developer_name("PictoTxt contributors")
        .license_type(gtk::License::MitX11)
        .comments("Drop or paste an image, copy the text.")
        .modal(true)
        .build();
    if let Some(parent) = app.active_window() {
        about.set_transient_for(Some(&parent));
    }
    about.present();
}

fn show_preferences(app: &adw::Application) {
    let win = adw::PreferencesWindow::builder()
        .title("Preferences")
        .modal(true)
        .build();
    if let Some(parent) = app.active_window() {
        win.set_transient_for(Some(&parent));
    }

    let page = adw::PreferencesPage::new();
    let group = adw::PreferencesGroup::builder().title("Behaviour").build();

    match load_settings() {
        Some(settings) => {
            let auto_copy = adw::SwitchRow::builder()
                .title("Auto-copy result")
                .build();
            let monospace = adw::SwitchRow::builder().title("Monospace text").build();
            settings.bind("auto-copy", &auto_copy, "active").build();
            settings.bind("monospace", &monospace, "active").build();
            group.add(&auto_copy);
            group.add(&monospace);
        }
        None => {
            group.set_description(Some(
                "Settings unavailable. Install the GNOME Shell extension first.",
            ));
        }
    }

    page.add(&group);
    win.add(&page);
    win.present();
}
