# ==============================================================================
# CCCP - Conventional Commits Compliance Program
# Makefile
# ==============================================================================

PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin
DESTDIR ?=
CARGO ?= cargo

TARGET ?=
CARGO_FLAGS = --release
ifneq ($(TARGET),)
	CARGO_FLAGS += --target $(TARGET)
endif

.PHONY: all build install uninstall test clean help

all: build

help:
	@echo "CCCP Build & Installation Targets:"
	@echo "  make build       Compile release binary"
	@echo "  make install     Compile and install binary to \$$(DESTDIR)\$$(BINDIR)"
	@echo "  make uninstall   Remove binary from \$$(DESTDIR)\$$(BINDIR)"
	@echo "  make test        Run test suite"
	@echo "  make clean       Clean build artifacts"

build:
	$(CARGO) build $(CARGO_FLAGS)

install: build
	@mkdir -p $(DESTDIR)$(BINDIR)
	@if [ -f "target/x86_64-unknown-linux-musl/release/cccp" ]; then \
		install -m 755 target/x86_64-unknown-linux-musl/release/cccp $(DESTDIR)$(BINDIR)/cccp; \
	elif [ -f "target/release/cccp" ]; then \
		install -m 755 target/release/cccp $(DESTDIR)$(BINDIR)/cccp; \
	else \
		CANDIDATE=$$(find target -type f -name cccp -path '*/release/cccp' 2>/dev/null | head -n 1); \
		if [ -n "$$CANDIDATE" ]; then \
			install -m 755 "$$CANDIDATE" $(DESTDIR)$(BINDIR)/cccp; \
		else \
			echo "Error: compiled binary not found in target/"; exit 1; \
		fi \
	fi
	@echo "Installed cccp to $(DESTDIR)$(BINDIR)/cccp"

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/cccp

test:
	$(CARGO) test

clean:
	$(CARGO) clean
