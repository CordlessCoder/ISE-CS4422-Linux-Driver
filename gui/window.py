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
                background-color: @theme_bg_color;
                color: @theme_fg_color;
            }
            .sidebar {
                background-color: @theme_base_color;
                border-right: 1px solid @borders;
            }
            .entry-row {
                padding: 10px 14px;
                border-bottom: 1px solid @borders;
            }
            .entry-row:hover {
                background-color: @theme_hover_color;
            }
            .entry-row.selected {
                background-color: @accent_bg_color;
            }
            .site-label {
                font-weight: bold;
                font-size: 14px;
                color: @theme_fg_color;
            }
            .username-label {
                font-size: 12px;
                color: @theme_fg_color;
                opacity: 0.7;
            }
            .detail-panel {
                background-color: @theme_base_color;
                border-left: 1px solid @borders;
            }
            .unlock-card {
                background-color: @theme_base_color;
                border-radius: 12px;
                padding: 32px;
            }
            .heading {
                font-size: 22px;
                font-weight: bold;
                color: @theme_fg_color;
            }
            .subheading {
                font-size: 14px;
                color: @theme_fg_color;
                opacity: 0.7;
            }
            .error-label {
                color: @error_color;
                font-size: 13px;
            }
            .password-hidden {
                font-family: monospace;
                letter-spacing: 4px;
                color: @theme_fg_color;
            }
            .tag {
                border-radius: 6px;
                padding: 2px 8px;
                font-size: 11px;
            }
            .tag-login {
                background-color: @accent_bg_color;
                color: @accent_fg_color;
            }
            .tag-card {
                background-color: @warning_color;
                opacity: 0.15;
                color: @theme_fg_color;
            }
            .tag-note {
                background-color: @suggested_color;
                opacity: 0.15;
                color: @theme_fg_color;
            }
            button.primary {
                background-color: @accent_color;
                color: @accent_fg_color;
                border-radius: 8px;
                padding: 8px 20px;
                border: none;
                font-weight: bold;
            }
            button.primary:hover {
                opacity: 0.9;
            }
            button.destructive {
                background-color: @error_color;
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