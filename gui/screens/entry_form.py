"""
screens/entry_form.py — Screen D
Used for both adding a new entry and editing an existing one.
"""

import gi
gi.require_version("Gtk", "4.0")
from gi.repository import Gtk

from datetime import date
import backend


CATEGORIES = ["Login", "Card", "Note"]


class EntryForm(Gtk.Box):
    def __init__(self, entry, on_save, on_cancel):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        self._entry = entry  # None if adding new
        self._on_save = on_save
        self._on_cancel = on_cancel
        self._dialog = None
        self.set_margin_top(20)
        self.set_margin_bottom(20)
        self.set_margin_start(20)
        self.set_margin_end(20)
        self._build()

    def set_dialog(self, dialog):
        """Give the form a reference to its parent dialog so it can close it."""
        self._dialog = dialog

    def _build(self):
        # Site / service name
        self.append(self._field_label("Site / service name"))
        self._site = Gtk.Entry()
        self._site.set_placeholder_text("e.g. GitHub")
        self.append(self._site)

        # Username
        self.append(self._field_label("Username / email"))
        self._username = Gtk.Entry()
        self._username.set_placeholder_text("e.g. you@example.com")
        self.append(self._username)

        # Password row with generator
        self.append(self._field_label("Password"))
        pass_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self._password = Gtk.Entry()
        self._password.set_visibility(False)
        self._password.set_hexpand(True)
        pass_row.append(self._password)

        gen_btn = Gtk.Button(label="Generate")
        gen_btn.connect("clicked", self._generate_password)
        pass_row.append(gen_btn)
        self.append(pass_row)

        # Category
        self.append(self._field_label("Category"))
        self._category = Gtk.DropDown.new_from_strings(CATEGORIES)
        self.append(self._category)

        # URL
        self.append(self._field_label("URL (optional)"))
        self._url = Gtk.Entry()
        self._url.set_placeholder_text("https://")
        self.append(self._url)

        # Notes
        self.append(self._field_label("Notes (optional)"))
        notes_scroll = Gtk.ScrolledWindow()
        notes_scroll.set_size_request(-1, 80)
        notes_scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        self._notes = Gtk.TextView()
        self._notes.set_wrap_mode(Gtk.WrapMode.WORD)
        notes_scroll.set_child(self._notes)
        self.append(notes_scroll)

        # Error label
        self._error = Gtk.Label()
        self._error.add_css_class("error-label")
        self._error.set_visible(False)
        self.append(self._error)

        # Buttons
        btn_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        btn_row.set_halign(Gtk.Align.END)

        cancel_btn = Gtk.Button(label="Cancel")
        cancel_btn.connect("clicked", self._cancel)
        btn_row.append(cancel_btn)

        save_btn = Gtk.Button(label="Save")
        save_btn.add_css_class("primary")
        save_btn.connect("clicked", self._save)
        btn_row.append(save_btn)

        self.append(btn_row)

        # Pre-populate if editing
        if self._entry:
            self._site.set_text(self._entry.get("site", ""))
            self._username.set_text(self._entry.get("username", ""))
            self._password.set_text(self._entry.get("password", ""))
            self._url.set_text(self._entry.get("url", ""))
            buf = self._notes.get_buffer()
            buf.set_text(self._entry.get("notes", ""))
            cat = self._entry.get("category", "Login")
            if cat in CATEGORIES:
                self._category.set_selected(CATEGORIES.index(cat))

    def _field_label(self, text: str) -> Gtk.Label:
        lbl = Gtk.Label(label=text)
        lbl.add_css_class("subheading")
        lbl.set_halign(Gtk.Align.START)
        return lbl

    def _generate_password(self, _):
        pwd = backend.generate_password(20)
        self._password.set_text(pwd)

    def _save(self, _):
        site = self._site.get_text().strip()
        if not site:
            self._error.set_label("Site name is required.")
            self._error.set_visible(True)
            return

        password = self._password.get_text()
        if not password:
            self._error.set_label("Password is required.")
            self._error.set_visible(True)
            return

        buf = self._notes.get_buffer()
        notes = buf.get_text(buf.get_start_iter(), buf.get_end_iter(), True)

        cat_index = self._category.get_selected()
        category = CATEGORIES[cat_index] if cat_index < len(CATEGORIES) else "Login"

        new_entry = {
            "id": self._entry["id"] if self._entry else backend.new_entry_id(),
            "site": site,
            "username": self._username.get_text().strip(),
            "password": password,
            "url": self._url.get_text().strip(),
            "notes": notes,
            "category": category,
            "modified": str(date.today()),
        }

        self._on_save(new_entry)
        if self._dialog:
            self._dialog.destroy()

    def _cancel(self, _):
        self._on_cancel()
        if self._dialog:
            self._dialog.destroy()