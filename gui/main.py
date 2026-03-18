#!/usr/bin/env python3
import gi
gi.require_version("Gtk", "4.0")
from gi.repository import Gtk

import sys
import os

import backend
from window import PassmanWindow


def main():
    app = Gtk.Application(application_id="com.passman.gui")

    def on_activate(application):
        win = PassmanWindow(application=application)
        win.present()

    app.connect("activate", on_activate)
    app.run(sys.argv)


if __name__ == "__main__":
    main()