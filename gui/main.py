#!/usr/bin/env python3
"""
main.py — entry point for the passman GUI.
Run with:  python3 main.py
Requires:  PyGObject with GTK4  (sudo pacman -S python-gobject gtk4)
"""

import gi
gi.require_version("Gtk", "4.0")
from gi.repository import Gtk, Gio

import backend
from window import PassmanWindow


class PassmanApp(Gtk.Application):
    def __init__(self):
        super().__init__(
            application_id="com.passman.gui",
            flags=Gio.ApplicationFlags.FLAGS_NONE,
        )

    def do_activate(self):
        win = PassmanWindow(application=self)
        win.present()


def main():
    app = PassmanApp()
    app.run()


if __name__ == "__main__":
    main()