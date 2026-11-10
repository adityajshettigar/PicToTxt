BIN   := extension/bin/picto-txt
UUID  := pictotxt@example.github.io
MODELS_DIR := rust/assets/models
DET_URL := https://huggingface.co/robertknight/ocrs/resolve/main/text-detection-ssfbcj81.rten
REC_URL := https://huggingface.co/robertknight/ocrs/resolve/main/text-rec-checkpoint-s52qdbqt.rten

.PHONY: build models schemas pack install clean

models:
	mkdir -p $(MODELS_DIR)
	curl -fL -o $(MODELS_DIR)/detection.rten $(DET_URL)
	curl -fL -o $(MODELS_DIR)/recognition.rten $(REC_URL)

build:
	@test "$$(stat -c%s $(MODELS_DIR)/detection.rten)" -gt 1000000 || $(MAKE) models
	@test "$$(stat -c%s $(MODELS_DIR)/recognition.rten)" -gt 1000000 || $(MAKE) models
	cd rust && cargo build --release
	mkdir -p extension/bin
	cp rust/target/release/picto-txt $(BIN)
	chmod +x $(BIN)

schemas:
	glib-compile-schemas extension/schemas

pack: build schemas
	gnome-extensions pack extension --force \
	  --extra-source=bin --extra-source=icons

install: pack
	gnome-extensions install --force $(UUID).shell-extension.zip
	glib-compile-schemas ~/.local/share/gnome-shell/extensions/$(UUID)/schemas

clean:
	cd rust && cargo clean
	rm -f $(BIN) *.shell-extension.zip extension/schemas/gschemas.compiled
