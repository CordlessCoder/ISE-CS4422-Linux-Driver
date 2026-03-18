"""
screens/vault.py — Screen C (main view) + Screen D (add/edit) + Screen E (detail)
Three-pane layout: category sidebar | entry list | detail panel
"""

import gi
gi.require_version("Gtk", "4.0")
from gi.repository import Gtk, Gdk, GLib

import backend
from screens.entry_form import EntryForm


CATEGORIES = ["All", "Login", "Card", "Note"]


class VaultScreen(Gtk.Box):
    def __init__(self, on_lock):
        super().__init__(orientation=Gtk.Orientation.HORIZONTAL)
        self._on_lock = on_lock
        self._state = None
        self._selected_entry = None
        self._current_category = "All"
        self._search_text = ""
        self._build()

    def load(self, state: dict):
        """Called by window.py after unlock/create with fresh state."""
        self._state = state
        self._refresh_list()

    # ── Build ──────────────────────────────────────────────────────────────────

    def _build(self):
        # ── Left sidebar: categories + lock button ────────────────────────────
        sidebar = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        sidebar.set_size_request(160, -1)
        sidebar.add_css_class("sidebar")

        app_title = Gtk.Label(label="Passman")
        app_title.set_margin_top(20)
        app_title.set_margin_bottom(12)
        app_title.set_margin_start(16)
        app_title.set_halign(Gtk.Align.START)
        app_title.add_css_class("site-label")
        sidebar.append(app_title)

        self._cat_buttons = {}
        for cat in CATEGORIES:
            btn = Gtk.Button(label=cat)
            btn.set_has_frame(False)
            btn.set_halign(Gtk.Align.FILL)
            btn.set_margin_start(8)
            btn.set_margin_end(8)
            btn.set_margin_top(2)
            btn.connect("clicked", self._on_category, cat)
            self._cat_buttons[cat] = btn
            sidebar.append(btn)

        spacer = Gtk.Box()
        spacer.set_vexpand(True)
        sidebar.append(spacer)

        lock_btn = Gtk.Button(label="🔒  Lock")
        lock_btn.set_has_frame(False)
        lock_btn.set_margin_start(8)
        lock_btn.set_margin_end(8)
        lock_btn.set_margin_bottom(16)
        lock_btn.connect("clicked", lambda _: self._on_lock())
        sidebar.append(lock_btn)

        self.append(sidebar)

        # ── Centre: search + entry list ───────────────────────────────────────
        centre = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        centre.set_hexpand(True)

        # Toolbar row
        toolbar = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        toolbar.set_margin_top(12)
        toolbar.set_margin_bottom(8)
        toolbar.set_margin_start(12)
        toolbar.set_margin_end(12)

        self._search = Gtk.SearchEntry()
        self._search.set_placeholder_text("Search entries…")
        self._search.set_hexpand(True)
        self._search.connect("search-changed", self._on_search)
        toolbar.append(self._search)

        add_btn = Gtk.Button(label="+ Add")
        add_btn.add_css_class("primary")
        add_btn.connect("clicked", self._on_add)
        toolbar.append(add_btn)

        centre.append(toolbar)

        # Scrollable entry list
        scroll = Gtk.ScrolledWindow()
        scroll.set_vexpand(True)
        scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)

        self._list_box = Gtk.ListBox()
        self._list_box.set_selection_mode(Gtk.SelectionMode.SINGLE)
        self._list_box.connect("row-selected", self._on_row_selected)
        scroll.set_child(self._list_box)
        centre.append(scroll)

        self.append(centre)

        # ── Right: detail panel ───────────────────────────────────────────────
        self._detail = DetailPanel(
            on_edit=self._on_edit,
            on_delete=self._on_delete,
        )
        self._detail.set_size_request(300, -1)
        self.append(self._detail)

    # ── List management ────────────────────────────────────────────────────────

    def _visible_entries(self):
        if not self._state:
            return []
        entries = self._state["entries"]
        if self._current_category != "All":
            entries = [e for e in entries if e.get("category") == self._current_category]
        if self._search_text:
            q = self._search_text.lower()
            entries = [
                e for e in entries
                if q in e.get("site", "").lower()
                or q in e.get("username", "").lower()
            ]
        return entries

    def _refresh_list(self):
        # Clear existing rows
        while (row := self._list_box.get_row_at_index(0)):
            self._list_box.remove(row)

        for entry in self._visible_entries():
            row = self._make_row(entry)
            self._list_box.append(row)

        self._detail.clear()

    def _make_row(self, entry: dict) -> Gtk.ListBoxRow:
        row = Gtk.ListBoxRow()
        row.entry_data = entry

        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        box.set_margin_top(10)
        box.set_margin_bottom(10)
        box.set_margin_start(14)
        box.set_margin_end(14)

        top_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)

        site = Gtk.Label(label=entry.get("site", "Untitled"))
        site.add_css_class("site-label")
        site.set_halign(Gtk.Align.START)
        site.set_hexpand(True)
        top_row.append(site)

        # Category tag
        cat = entry.get("category", "Login")
        tag = Gtk.Label(label=cat)
        tag.add_css_class("tag")
        tag.add_css_class(f"tag-{cat.lower()}")
        top_row.append(tag)

        box.append(top_row)

        username = Gtk.Label(label=entry.get("username", ""))
        username.add_css_class("username-label")
        username.set_halign(Gtk.Align.START)
        box.append(username)

        row.set_child(box)
        return row

    # ── Event handlers ─────────────────────────────────────────────────────────

    def _on_category(self, _, category: str):
        self._current_category = category
        self._refresh_list()

    def _on_search(self, entry):
        self._search_text = entry.get_text()
        self._refresh_list()

    def _on_row_selected(self, _, row):
        if row is None:
            self._detail.clear()
            return
        self._selected_entry = row.entry_data
        self._detail.show_entry(row.entry_data)

    def _on_add(self, _):
        self._open_form(None)

    def _on_edit(self, entry: dict):
        self._open_form(entry)

    def _on_delete(self, entry: dict):
        dialog = Gtk.MessageDialog(
            transient_for=self.get_root(),
            modal=True,
            message_type=Gtk.MessageType.WARNING,
            buttons=Gtk.ButtonsType.CANCEL,
            text=f"Delete \"{entry['site']}\"?",
        )
        dialog.format_secondary_text("This cannot be undone.")
        dialog.add_button("Delete", Gtk.ResponseType.ACCEPT)
        dialog.connect("response", self._on_delete_response, entry)
        dialog.present()

    def _on_delete_response(self, dialog, response, entry):
        dialog.destroy()
        if response == Gtk.ResponseType.ACCEPT:
            self._state["entries"] = [
                e for e in self._state["entries"] if e["id"] != entry["id"]
            ]
            backend.save(self._state["passphrase"], self._state["entries"])
            self._detail.clear()
            self._refresh_list()

    def _open_form(self, entry):
        form = EntryForm(
            entry=entry,
            on_save=self._on_form_save,
            on_cancel=lambda: None,
        )
        dialog = Gtk.Window(title="Edit entry" if entry else "New entry")
        dialog.set_transient_for(self.get_root())
        dialog.set_modal(True)
        dialog.set_default_size(420, 520)
        dialog.set_child(form)
        form.set_dialog(dialog)
        dialog.present()

    def _on_form_save(self, new_entry: dict):
        entries = self._state["entries"]
        existing = next((e for e in entries if e["id"] == new_entry["id"]), None)
        if existing:
            entries[entries.index(existing)] = new_entry
        else:
            entries.append(new_entry)
        backend.save(self._state["passphrase"], entries)
        self._refresh_list()


# ── Detail panel ──────────────────────────────────────────────────────────────

class DetailPanel(Gtk.Box):
    def __init__(self, on_edit, on_delete):
        super().__init__(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self._on_edit = on_edit
        self._on_delete = on_delete
        self._entry = None
        self._password_visible = False
        self._clipboard_timer = None
        self.add_css_class("detail-panel")
        self._build()

    def _build(self):
        self._stack = Gtk.Stack()
        self._stack.set_vexpand(True)

        # Empty state
        empty = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        empty.set_valign(Gtk.Align.CENTER)
        empty.set_halign(Gtk.Align.CENTER)
        lbl = Gtk.Label(label="Select an entry")
        lbl.add_css_class("subheading")
        empty.append(lbl)
        self._stack.add_named(empty, "empty")

        # Detail state
        detail_scroll = Gtk.ScrolledWindow()
        detail_scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        detail_content = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        detail_content.set_margin_top(20)
        detail_content.set_margin_bottom(20)
        detail_content.set_margin_start(20)
        detail_content.set_margin_end(20)

        # Site name
        self._site_label = Gtk.Label()
        self._site_label.add_css_class("heading")
        self._site_label.set_halign(Gtk.Align.START)
        self._site_label.set_wrap(True)
        detail_content.append(self._site_label)

        # Category tag
        self._cat_tag = Gtk.Label()
        self._cat_tag.set_halign(Gtk.Align.START)
        detail_content.append(self._cat_tag)

        detail_content.append(Gtk.Separator())

        # Fields
        self._username_row = self._make_field("Username", copyable=True)
        detail_content.append(self._username_row["box"])

        self._password_row = self._make_password_field()
        detail_content.append(self._password_row["box"])

        self._url_row = self._make_field("URL", copyable=False)
        detail_content.append(self._url_row["box"])

        self._notes_row = self._make_field("Notes", copyable=False)
        detail_content.append(self._notes_row["box"])

        self._modified_label = Gtk.Label()
        self._modified_label.add_css_class("subheading")
        self._modified_label.set_halign(Gtk.Align.START)
        detail_content.append(self._modified_label)

        detail_content.append(Gtk.Separator())

        # Action buttons
        btn_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        edit_btn = Gtk.Button(label="Edit")
        edit_btn.add_css_class("primary")
        edit_btn.connect("clicked", lambda _: self._on_edit(self._entry))
        btn_row.append(edit_btn)

        del_btn = Gtk.Button(label="Delete")
        del_btn.add_css_class("destructive")
        del_btn.connect("clicked", lambda _: self._on_delete(self._entry))
        btn_row.append(del_btn)

        detail_content.append(btn_row)

        # Clipboard notice
        self._clip_label = Gtk.Label()
        self._clip_label.add_css_class("subheading")
        self._clip_label.set_visible(False)
        detail_content.append(self._clip_label)

        detail_scroll.set_child(detail_content)
        self._stack.add_named(detail_scroll, "detail")

        self._stack.set_visible_child_name("empty")
        self.append(self._stack)

    def _make_field(self, label_text: str, copyable: bool) -> dict:
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        label = Gtk.Label(label=label_text)
        label.add_css_class("subheading")
        label.set_halign(Gtk.Align.START)
        box.append(label)
        value = Gtk.Label()
        value.set_halign(Gtk.Align.START)
        value.set_selectable(True)
        value.set_wrap(True)
        box.append(value)
        return {"box": box, "value": value}

    def _make_password_field(self) -> dict:
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        label = Gtk.Label(label="Password")
        label.add_css_class("subheading")
        label.set_halign(Gtk.Align.START)
        box.append(label)

        row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        value = Gtk.Label(label="••••••••••••")
        value.set_halign(Gtk.Align.START)
        value.set_hexpand(True)
        row.append(value)

        reveal_btn = Gtk.Button(label="Show")
        reveal_btn.set_has_frame(False)
        reveal_btn.connect("clicked", self._toggle_password)
        row.append(reveal_btn)

        copy_btn = Gtk.Button(label="Copy")
        copy_btn.set_has_frame(False)
        copy_btn.connect("clicked", self._copy_password)
        row.append(copy_btn)

        box.append(row)
        return {"box": box, "value": value, "reveal_btn": reveal_btn}

    def show_entry(self, entry: dict):
        self._entry = entry
        self._password_visible = False

        self._site_label.set_label(entry.get("site", ""))
        cat = entry.get("category", "Login")
        self._cat_tag.set_label(cat)
        self._cat_tag.set_css_classes(["tag", f"tag-{cat.lower()}"])

        self._username_row["value"].set_label(entry.get("username", "—"))
        self._password_row["value"].set_label("••••••••••••")
        self._password_row["reveal_btn"].set_label("Show")

        url = entry.get("url", "")
        self._url_row["value"].set_label(url if url else "—")

        notes = entry.get("notes", "")
        self._notes_row["value"].set_label(notes if notes else "—")

        self._modified_label.set_label(f"Modified {entry.get('modified', '—')}")
        self._clip_label.set_visible(False)

        self._stack.set_visible_child_name("detail")

    def clear(self):
        self._entry = None
        self._stack.set_visible_child_name("empty")

    def _toggle_password(self, btn):
        if not self._entry:
            return
        self._password_visible = not self._password_visible
        if self._password_visible:
            self._password_row["value"].set_label(self._entry.get("password", ""))
            btn.set_label("Hide")
        else:
            self._password_row["value"].set_label("••••••••••••")
            btn.set_label("Show")

    def _copy_password(self, _):
        if not self._entry:
            return
        password = self._entry.get("password", "")
        clipboard = Gdk.Display.get_default().get_clipboard()
        clipboard.set(password)

        self._clip_label.set_label("Password copied — clears in 30s")
        self._clip_label.set_visible(True)

        if self._clipboard_timer:
            GLib.source_remove(self._clipboard_timer)
        self._clipboard_timer = GLib.timeout_add_seconds(30, self._clear_clipboard)

    def _clear_clipboard(self):
        clipboard = Gdk.Display.get_default().get_clipboard()
        clipboard.set("")
        self._clip_label.set_label("Clipboard cleared.")
        GLib.timeout_add_seconds(3, lambda: self._clip_label.set_visible(False) or False)
        self._clipboard_timer = None
        return False