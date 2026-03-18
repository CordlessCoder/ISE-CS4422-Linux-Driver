"""
screens/unlock.py — Screen B
Shown when a vault already exists. User enters their master passphrase.
"""

import gi
gi.require_version("Gtk", "4.0")
from gi.repository import Gtk, GLib

import backend


class UnlockScreen(Gtk.Box):
    def __init__(self, on_unlock):
        super().__init__(orientation=Gtk.Orientation.VERTICAL)
        self._on_unlock = on_unlock
        self._build()

    def _build(self):
        # Centre everything vertically and horizontally
        self.set_valign(Gtk.Align.CENTER)
        self.set_halign(Gtk.Align.CENTER)
        self.set_spacing(0)

        # Card container
        card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=20)
        card.set_size_request(360, -1)
        card.add_css_class("unlock-card")

        # Lock icon (unicode, no external assets needed)
        icon = Gtk.Label(label="🔒")
        icon.set_css_classes(["heading"])
        card.append(icon)

        # Title
        title = Gtk.Label(label="Passman")
        title.add_css_class("heading")
        card.append(title)

        subtitle = Gtk.Label(label="Enter your master passphrase to unlock")
        subtitle.add_css_class("subheading")
        subtitle.set_wrap(True)
        subtitle.set_justify(Gtk.Justification.CENTER)
        card.append(subtitle)

        # Passphrase field
        self._passphrase_entry = Gtk.PasswordEntry()
        self._passphrase_entry.set_show_peek_icon(True)
        self._passphrase_entry.set_placeholder_text("Master passphrase")
        self._passphrase_entry.connect("activate", self._on_submit)
        card.append(self._passphrase_entry)

        # Error label (hidden until needed)
        self._error_label = Gtk.Label(label="Incorrect passphrase, try again.")
        self._error_label.add_css_class("error-label")
        self._error_label.set_visible(False)
        card.append(self._error_label)

        # Unlock button
        unlock_btn = Gtk.Button(label="Unlock vault")
        unlock_btn.add_css_class("primary")
        unlock_btn.connect("clicked", self._on_submit)
        card.append(unlock_btn)

        self.append(card)

    def reset(self):
        """Clear the passphrase field and error when returning to this screen."""
        self._passphrase_entry.set_text("")
        self._error_label.set_visible(False)

    def _on_submit(self, *_):
        passphrase = self._passphrase_entry.get_text()
        if not passphrase:
            return

        entries = backend.unlock(passphrase)
        if entries is None:
            self._error_label.set_visible(True)
            self._passphrase_entry.set_text("")
            return

        self._error_label.set_visible(False)
        self._on_unlock(passphrase, entries)