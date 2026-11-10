import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';
import St from 'gi://St';

import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';

const BINARY_NAME = 'picto-txt';
const SHORTCUT_KEY = 'open-shortcut';

export default class PictoTxtExtension extends Extension {
    enable() {
        this._settings = this.getSettings();
        this._settingsHandlers = [];
        this._keybindingRegistered = false;
        this._label = null;

        // Menu-less panel button; the primary click launches the app.
        this._button = new PanelMenu.Button(0.5, this.metadata.name, true);

        this._box = new St.BoxLayout({style_class: 'panel-status-menu-box'});
        this._icon = new St.Icon({style_class: 'system-status-icon'});
        this._box.add_child(this._icon);
        this._button.add_child(this._box);

        this._button.connect('button-press-event', (_actor, event) => {
            if (event.get_button() !== Clutter.BUTTON_PRIMARY)
                return Clutter.EVENT_PROPAGATE;
            this._launch();
            return Clutter.EVENT_STOP;
        });
        this._button.connect('touch-event', (_actor, event) => {
            if (event.type() !== Clutter.EventType.TOUCH_BEGIN)
                return Clutter.EVENT_PROPAGATE;
            this._launch();
            return Clutter.EVENT_STOP;
        });

        this._updateIcon();
        this._updateLabel();
        this._updateKeybinding();

        for (const [key, handler] of [
            ['icon-name', () => this._updateIcon()],
            ['show-label', () => this._updateLabel()],
            ['label-text', () => this._updateLabel()],
            [SHORTCUT_KEY, () => this._updateKeybinding()],
        ]) {
            const id = this._settings.connect(`changed::${key}`, handler);
            this._settingsHandlers.push(id);
        }

        Main.panel.addToStatusArea(this.uuid, this._button);
    }

    disable() {
        if (this._settings && this._settingsHandlers) {
            for (const id of this._settingsHandlers)
                this._settings.disconnect(id);
        }
        this._settingsHandlers = [];

        this._removeKeybinding();

        this._button?.destroy();
        this._button = null;
        this._box = null;
        this._icon = null;
        this._label = null;
        this._settings = null;
    }

    _defaultIconPath() {
        return this.dir.get_child('icons').get_child('picto-txt-symbolic.svg').get_path();
    }

    _updateIcon() {
        if (!this._icon || !this._settings)
            return;

        let name = this._settings.get_string('icon-name').trim();
        if (name === '')
            name = this._defaultIconPath();

        let gicon = null;
        try {
            gicon = Gio.icon_new_for_string(name);
        } catch (e) {
            console.warn(`PictoTxt: invalid icon "${name}": ${e.message}`);
        }
        if (gicon === null)
            gicon = Gio.icon_new_for_string(this._defaultIconPath());

        this._icon.gicon = gicon;
    }

    _updateLabel() {
        if (!this._box || !this._settings)
            return;

        const show = this._settings.get_boolean('show-label');
        const text = this._settings.get_string('label-text');

        if (!show) {
            if (this._label) {
                this._label.destroy();
                this._label = null;
            }
            return;
        }

        if (!this._label) {
            this._label = new St.Label({
                text,
                y_align: Clutter.ActorAlign.CENTER,
                style_class: 'pictotxt-label',
            });
            this._box.add_child(this._label);
        } else {
            this._label.text = text;
        }
    }

    _removeKeybinding() {
        if (this._keybindingRegistered) {
            Main.wm.removeKeybinding(SHORTCUT_KEY);
            this._keybindingRegistered = false;
        }
    }

    _updateKeybinding() {
        if (!this._settings)
            return;

        this._removeKeybinding();

        const accels = this._settings.get_strv(SHORTCUT_KEY);
        if (accels.length === 0 || accels[0].trim() === '')
            return;

        try {
            Main.wm.addKeybinding(
                SHORTCUT_KEY,
                this._settings,
                Meta.KeyBindingFlags.NONE,
                Shell.ActionMode.NORMAL | Shell.ActionMode.OVERVIEW,
                () => this._launch());
            this._keybindingRegistered = true;
        } catch (e) {
            console.warn(`PictoTxt: could not register shortcut: ${e.message}`);
        }
    }

    _findBinary() {
        const candidates = [
            GLib.build_filenamev([GLib.get_home_dir(), '.local', 'bin', BINARY_NAME]),
            GLib.build_filenamev([this.dir.get_path(), 'bin', BINARY_NAME]),
        ];
        for (const path of candidates) {
            if (GLib.file_test(path, GLib.FileTest.IS_EXECUTABLE))
                return path;
        }
        return null;
    }

    _launch() {
        const path = this._findBinary();
        if (path === null) {
            Main.notifyError('PictoTxt', 'Rust binary not found. Reinstall the extension.');
            this.openPreferences();
            return;
        }

        try {
            Gio.Subprocess.new(
                [path],
                Gio.SubprocessFlags.STDOUT_SILENCE | Gio.SubprocessFlags.STDERR_SILENCE);
        } catch (e) {
            Main.notifyError('PictoTxt', `Could not start PictoTxt: ${e.message}`);
        }
    }
}
