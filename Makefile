PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
MANDIR ?= $(PREFIX)/share/man/man1
BASH_COMPLETION_DIR ?= /etc/bash_completion.d
ZSH_COMPLETION_DIR ?= $(PREFIX)/share/zsh/site-functions
FISH_COMPLETION_DIR ?= $(PREFIX)/share/fish/vendor_completions.d
DESKTOP_DIR ?= $(PREFIX)/share/applications
ICON_DIR ?= $(PREFIX)/share/icons/hicolor/scalable/apps

.PHONY: all build build-gui test check install install-gui uninstall uninstall-gui clean

all: build

# Builds ONLY the dependency-free kbdanim CLI/daemon -- no GUI
# toolkit required. This is the default and is what most users and
# packagers want.
build:
	cargo build --release

# Builds kbdanim-gui too. Requires the `gui` feature, which pulls in
# eframe/egui (see Cargo.toml and docs/gui.md) -- only touched if you
# actually run this target.
build-gui:
	cargo build --release --features gui --bin kbdanim-gui

test:
	cargo test

check:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo test

install: build
	install -Dm755 target/release/kbdanim $(DESTDIR)$(BINDIR)/kbdanim
	install -Dm644 man/kbdanim.1 $(DESTDIR)$(MANDIR)/kbdanim.1
	install -Dm644 completions/kbdanim.bash $(DESTDIR)$(BASH_COMPLETION_DIR)/kbdanim
	install -Dm644 completions/kbdanim.zsh $(DESTDIR)$(ZSH_COMPLETION_DIR)/_kbdanim
	install -Dm644 completions/kbdanim.fish $(DESTDIR)$(FISH_COMPLETION_DIR)/kbdanim.fish
	@echo ""
	@echo "kbdanim installed to $(DESTDIR)$(BINDIR)/kbdanim"
	@echo "Next steps (optional but recommended):"
	@echo "  sudo cp udev/60-kbdanim.rules /etc/udev/rules.d/ && sudo udevadm control --reload-rules"
	@echo "  mkdir -p ~/.config/systemd/user && cp systemd/kbdanim.service ~/.config/systemd/user/"
	@echo "  systemctl --user daemon-reload && systemctl --user enable --now kbdanim"
	@echo "Want the GUI too? Run: make install-gui"

# Installs the GUI binary, its desktop entry, and its icon. Does NOT
# build/install the CLI -- run `make install` first (or `make
# install-gui` after it) so kbdanim-gui can find its sibling `kbdanim`
# binary to launch the daemon (see src/gui/backend.rs).
install-gui: build-gui
	install -Dm755 target/release/kbdanim-gui $(DESTDIR)$(BINDIR)/kbdanim-gui
	install -Dm644 assets/kbdanim.desktop $(DESTDIR)$(DESKTOP_DIR)/kbdanim.desktop
	install -Dm644 assets/kbdanim.svg $(DESTDIR)$(ICON_DIR)/kbdanim.svg
	@echo ""
	@echo "kbdanim-gui installed to $(DESTDIR)$(BINDIR)/kbdanim-gui"
	@echo "It should now appear in your application menu as \"kbdanim\"."

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/kbdanim
	rm -f $(DESTDIR)$(MANDIR)/kbdanim.1
	rm -f $(DESTDIR)$(BASH_COMPLETION_DIR)/kbdanim
	rm -f $(DESTDIR)$(ZSH_COMPLETION_DIR)/_kbdanim
	rm -f $(DESTDIR)$(FISH_COMPLETION_DIR)/kbdanim.fish

uninstall-gui:
	rm -f $(DESTDIR)$(BINDIR)/kbdanim-gui
	rm -f $(DESTDIR)$(DESKTOP_DIR)/kbdanim.desktop
	rm -f $(DESTDIR)$(ICON_DIR)/kbdanim.svg

clean:
	cargo clean
