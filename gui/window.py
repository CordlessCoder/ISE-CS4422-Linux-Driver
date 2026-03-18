"""
window.py — the top-level window.

Uses a Gtk.Stack to swap between screens:
  "create"  → CreateVaultScreen  (first run only)
  "unlock"  → UnlockScreen
  "vault"   → VaultScreen        (main app)
"""

import gi
gi.require_version("Gtk", "4.0")
from gi.repository import Gtk

import backend
from screens.create_vault import CreateVaultScreen
from screens.unlock import UnlockScreen
from screens.vault import VaultScreen


class PassmanWindow(Gtk.ApplicationWindow):
    def __init__(self, **kwargs):
        super().__init__(
            title="Passman",
            default_width=900,
            default_height=600,
            **kwargs,
        )

        # ── CSS ───────────────────────────────────────────────────────────────
        css = Gtk.CssProvider()
        css.load_from_data(b"""
            window {
                background-color: #f6f5f4;
            }
            .sidebar {
                background-color: #ffffff;
                border-right: 1px solid #e0dedd;
            }
            .entry-row {
                padding: 10px 14px;
                border-bottom: 1px solid #f0efee;
            }
            .entry-row:hover {
                background-color: #f0efee;
            }
            .entry-row.selected {
                background-color: #e8e7ff;
            }
            .site-label {
                font-weight: bold;
                font-size: 14px;
            }
            .username-label {
                font-size: 12px;
                color: #888;
            }
            .detail-panel {
                background-color: #ffffff;
                border-left: 1px solid #e0dedd;
            }
            .unlock-card {
                background-color: #ffffff;
                border-radius: 12px;
                padding: 32px;
            }
            .heading {
                font-size: 22px;
                font-weight: bold;
            }
            .subheading {
                font-size: 14px;
                color: #888;
            }
            .error-label {
                color: #c0392b;
                font-size: 13px;
            }
            .password-hidden {
                font-family: monospace;
                letter-spacing: 4px;
            }
            .tag {
                border-radius: 6px;
                padding: 2px 8px;
                font-size: 11px;
            }
            .tag-login {
                background-color: #e8e7ff;
                color: #534ab7;
            }
            .tag-card {
                background-color: #faeeda;
                color: #854f0b;
            }
            .tag-note {
                background-color: #e1f5ee;
                color: #0f6e56;
            }
            button.primary {
                background-color: #534ab7;
                color: white;
                border-radius: 8px;
                padding: 8px 20px;
                border: none;
                font-weight: bold;
            }
            button.primary:hover {
                background-color: #3c3489;
            }
            button.destructive {
                background-color: #c0392b;
                color: white;
                border-radius: 8px;
                padding: 8px 20px;
                border: none;
            }
        """)
        Gtk.StyleContext.add_provider_for_display(
            self.get_display(),
            css,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION,
        )

        # ── Stack (screen router) ─────────────────────────────────────────────
        self.stack = Gtk.Stack()
        self.stack.set_transition_type(Gtk.StackTransitionType.CROSSFADE)
        self.stack.set_transition_duration(150)
        self.set_child(self.stack)

        # Shared app state passed between screens
        self.state = {
            "passphrase": None,
            "entries": [],
        }

        # Build all screens
        self._create_screen = CreateVaultScreen(on_done=self._after_create)
        self._unlock_screen = UnlockScreen(on_unlock=self._after_unlock)
        self._vault_screen = VaultScreen(on_lock=self._do_lock)

        self.stack.add_named(self._create_screen, "create")
        self.stack.add_named(self._unlock_screen, "unlock")
        self.stack.add_named(self._vault_screen, "vault")

        # Route to first screen
        if backend.vault_exists():
            self.stack.set_visible_child_name("unlock")
        else:
            self.stack.set_visible_child_name("create")

    # ── Callbacks ─────────────────────────────────────────────────────────────

    def _after_create(self, passphrase: str):
        """Called by CreateVaultScreen when vault is created successfully."""
        self.state["passphrase"] = passphrase
        self.state["entries"] = []
        self._vault_screen.load(self.state)
        self.stack.set_visible_child_name("vault")

    def _after_unlock(self, passphrase: str, entries: list):
        """Called by UnlockScreen when passphrase is correct."""
        self.state["passphrase"] = passphrase
        self.state["entries"] = entries
        self._vault_screen.load(self.state)
        self.stack.set_visible_child_name("vault")

    def _do_lock(self):
        """Called by VaultScreen when user clicks Lock."""
        self.state["passphrase"] = None
        self.state["entries"] = []
        self._unlock_screen.reset()
        self.stack.set_visible_child_name("unlock")