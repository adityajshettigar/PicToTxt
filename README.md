# PictoTxt

A lightweight GNOME top-bar OCR utility. Click the icon, drop or paste an image, copy the text.

- **extension/**: GNOME Shell extension (GJS, ES modules) that launches the app.
- **rust/**: GTK4 + libadwaita app (`picto-txt`) that runs OCR with the pure-Rust [`ocrs`](https://github.com/robertknight/ocrs) engine. No network access at runtime.

<!-- TODO: add screenshots here -->

## Screenshots

TODO

## Requirements

Required system packages (names as on Fedora; use the equivalents on other distributions):

- `cargo`
- `gtk4-devel`
- `libadwaita-devel`
- `glib2-devel`
- `glib-compile-schemas` (from `glib2-devel` / `libglib2.0-bin`)
- `curl` (to fetch the OCR models)

GNOME Shell 45, 46 or 47. Wayland and X11 are supported.

## Build and install

```
make models    # fetch the OCR models (done automatically by `make build` if missing)
make install
```

Targets: `build`, `models`, `schemas`, `pack`, `install`, `clean`.

## OCR models

The two `.rten` model files are not stored in this repository; `make models` downloads them into `rust/assets/models/` as `detection.rten` and `recognition.rten`:

- `https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten`
- `https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten`

Model release tag: TODO. Record the exact `ocrs-models` release (or file checksums) you built against here for reproducible builds.
The shipped placeholder files in `rust/assets/models/` are only there so the tree is complete; replace them with the real models before building.

## Crate versions

`rust/Cargo.toml` pins the versions listed in the project brief (`gtk4 0.9`, `libadwaita 0.7`, `ocrs 0.6`, `rten 0.20`). These were not compiled or verified when this tree was generated. `ocrs` must use the same `rten` version as the one listed in `Cargo.toml`; adjust both together and record the exact versions here after your first successful build.

TODO: record verified versions.

## Binary size

TODO: record the size of `extension/bin/picto-txt` after `cargo build --release` (target: 35 MB or less).

## Restarting the shell

- Wayland: log out and back in.
- X11: press `Alt+F2`, type `r`, press Enter.

Then enable the extension with `gnome-extensions enable pictotxt@example.github.io`.

## Custom top-bar icon

Open the extension preferences. In **Top bar icon**, enter either an icon theme name (for example `edit-paste-symbolic`) or the absolute path to a `*-symbolic.svg` file. The browse button opens a file chooser for SVG files. The icon updates live.

## Keyboard shortcut

In the preferences, set **Open shortcut** to an accelerator such as `<Super>o` and press the apply button. Leave it empty to disable the shortcut. The setting is stored as a string array (`as`) because GNOME Shell keybindings require that type.

## Usage

- Drop an image on the window, press `Ctrl+V` to paste an image or an image file, or press `Ctrl+O` to choose a file.
- **Copy** copies the whole result. `Ctrl+C` copies the selection, or everything if nothing is selected.
- **Clear** resets the window. `Escape` closes it.

## Known limitations

- `ocrs` supports English only.
- Batch OCR is not supported; only the first dropped image is processed.
- Wayland clipboard image paste depends on the source app exposing `image/png` or a file URI.
