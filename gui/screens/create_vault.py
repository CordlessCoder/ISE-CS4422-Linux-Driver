"""
screens/create_vault.py — Screen A
Shown only on first run (no vault found).
"""

import gi
gi.require_version("Gtk", "4.0")
from gi.repository import Gtk

import backend


class CreateVaultScreen(Gtk.Box):
    def __init__(self, on_done):
        super().__init__(orientation=Gtk.Orientation.VERTICAL)
        self._on_done = on_done
        self._build()

    def _build(self):
        self.set_valign(Gtk.Align.CENTER)
        self.set_halign(Gtk.Align.CENTER)

        card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=20)
        card.set_size_request(380, -1)
        card.add_css_class("unlock-card")

        icon = Gtk.Label(label="🔐")
        icon.add_css_class("heading")
        card.append(icon)

        title = Gtk.Label(label="Create your vault")
        title.add_css_class("heading")
        card.append(title)

        subtitle = Gtk.Label(
            label="Choose a strong master passphrase.\nYou'll need this every time you open Passman."
        )
        subtitle.add_css_class("subheading")
        subtitle.set_wrap(True)
        subtitle.set_justify(Gtk.Justification.CENTER)
        card.append(subtitle)

        # Passphrase
        self._pass1 = Gtk.Entry()
        self._pass1.set_visibility(False)
        self._pass1.set_placeholder_text("Master passphrase")
        card.append(self._pass1)

        self._pass2 = Gtk.Entry()
        self._pass2.set_visibility(False)
        self._pass2.set_placeholder_text("Confirm passphrase")
        self._pass2.connect("activate", self._on_submit)
        card.append(self._pass2)

        # Error
        self._error = Gtk.Label()
        self._error.add_css_class("error-label")
        self._error.set_visible(False)
        card.append(self._error)

        btn = Gtk.Button(label="Create vault")
        btn.add_css_class("primary")
        btn.connect("clicked", self._on_submit)
        card.append(btn)

        self.append(card)

    def _on_submit(self, *_):
        p1 = self._pass1.get_text()
        p2 = self._pass2.get_text()

        if not p1:
            self._show_error("Please enter a passphrase.")
            return
        if len(p1) < 8:
            self._show_error("Passphrase must be at least 8 characters.")
            return
        if p1 != p2:
            self._show_error("Passphrases do not match.")
            return

        ok = backend.create_vault(p1)
        if not ok:
            self._show_error("Failed to create vault. Is the driver loaded?")
            return

        self._on_done(p1)

    def _show_error(self, msg: str):
        self._error.set_label(msg)
        self._error.set_visible(True)