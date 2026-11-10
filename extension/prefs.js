import Adw from 'gi://Adw';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';
import Gio from 'gi://Gio';
import Gtk from 'gi://Gtk';

import {ExtensionPreferences} from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

export default class PictoTxtPrefs extends ExtensionPreferences {
    fillPreferencesWindow(window) {
        const settings = this.getSettings();
        window._settings = settings;

        const page = new Adw.PreferencesPage({
            title: 'PictoTxt',
            icon_name: 'preferences-system-symbolic',
        });
        window.add(page);

        // ---- Appearance ----
        const appearance = new Adw.PreferencesGroup({title: 'Appearance'});
        page.add(appearance);

        const iconRow = new Adw.EntryRow({title: 'Top bar icon'});
        const browseButton = new Gtk.Button({
            icon_name: 'document-open-symbolic',
            valign: Gtk.Align.CENTER,
            tooltip_text: 'Choose an SVG icon',
        });
        browseButton.add_css_class('flat');
        browseButton.connect('clicked', () => {
            const filter = new Gtk.FileFilter();
            filter.set_name('SVG images');
            filter.add_mime_type('image/svg+xml');
            const filters = new Gio.ListStore({item_type: Gtk.FileFilter});
            filters.append(filter);

            const dialog = new Gtk.FileDialog({
                title: 'Choose a symbolic SVG icon',
                modal: true,
                filters,
                default_filter: filter,
            });
            dialog.open(window, null, (source, result) => {
                try {
                    const file = source.open_finish(result);
                    const path = file?.get_path();
                    if (path)
                        iconRow.set_text(path);
                } catch (e) {
                    // Dialog dismissed: nothing to do.
                }
            });
        });
        iconRow.add_suffix(browseButton);
        appearance.add(iconRow);
        settings.bind('icon-name', iconRow, 'text', Gio.SettingsBindFlags.DEFAULT);

        const hintRow = new Adw.ActionRow({
            title: 'Use an icon theme name (e.g. edit-paste-symbolic) or an absolute path to a *-symbolic.svg.',
            activatable: false,
            selectable: false,
        });
        hintRow.add_css_class('dim-label');
        hintRow.add_css_class('caption');
        appearance.add(hintRow);

        const showLabelRow = new Adw.SwitchRow({title: 'Show label next to icon'});
        appearance.add(showLabelRow);
        settings.bind('show-label', showLabelRow, 'active', Gio.SettingsBindFlags.DEFAULT);

        const labelRow = new Adw.EntryRow({title: 'Label text'});
        appearance.add(labelRow);
        settings.bind('label-text', labelRow, 'text', Gio.SettingsBindFlags.DEFAULT);
        showLabelRow.bind_property('active', labelRow, 'sensitive', GObject.BindingFlags.SYNC_CREATE);

        // ---- Behaviour ----
        const behaviour = new Adw.PreferencesGroup({title: 'Behaviour'});
        page.add(behaviour);

        const autoCopyRow = new Adw.SwitchRow({
            title: 'Auto-copy result',
            subtitle: 'Copy extracted text to the clipboard automatically',
        });
        behaviour.add(autoCopyRow);
        settings.bind('auto-copy', autoCopyRow, 'active', Gio.SettingsBindFlags.DEFAULT);

        const monoRow = new Adw.SwitchRow({title: 'Monospace text'});
        behaviour.add(monoRow);
        settings.bind('monospace', monoRow, 'active', Gio.SettingsBindFlags.DEFAULT);

        // ---- Shortcut ----
        const shortcut = new Adw.PreferencesGroup({title: 'Shortcut'});
        page.add(shortcut);

        const shortcutRow = new Adw.EntryRow({
            title: 'Open shortcut (e.g. <Super>o, empty to disable)',
            show_apply_button: true,
        });
        shortcut.add(shortcutRow);

        const storedShortcut = () => {
            const accels = settings.get_strv('open-shortcut');
            return accels.length > 0 ? accels[0] : '';
        };
        shortcutRow.set_text(storedShortcut());

        shortcutRow.connect('apply', () => {
            const text = shortcutRow.get_text().trim();

            if (text === '') {
                settings.set_strv('open-shortcut', []);
                return;
            }

            const [ok, key] = Gtk.accelerator_parse(text);
            if (ok && key !== 0) {
                settings.set_strv('open-shortcut', [text]);
                return;
            }

            // Invalid accelerator: revert and flash a warning.
            shortcutRow.set_text(storedShortcut());
            shortcutRow.add_css_class('error');
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1500, () => {
                shortcutRow.remove_css_class('error');
                return GLib.SOURCE_REMOVE;
            });
            window.add_toast(new Adw.Toast({title: 'Invalid shortcut'}));
        });
    }
}
