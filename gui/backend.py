"""
backend.py — all communication with passman-cli lives here.

When Dillion's CLI is ready, replace the STUB functions with real
subprocess.run() calls. The GUI code never changes — only this file.

CLI contract agreed with Dillion:
  passman-cli create-vault --vault <path>        (passphrase via stdin)
  passman-cli unlock        --vault <path>        (passphrase via stdin) -> JSON
  passman-cli save          --vault <path>        (passphrase + JSON via stdin)
  passman-cli gen-password  --length <n>          -> plain string
  All return exit code 0 on success, 1 on failure.
"""

import json
import os
import subprocess
import uuid
from pathlib import Path

VAULT_PATH = Path.home() / ".config" / "passman" / "vault.json"

# ── Stub data ──────────────────────────────────────────────────────────────────
_MOCK_ENTRIES = [
    {
        "id": "1",
        "site": "GitHub",
        "username": "adam@example.com",
        "password": "hunter2_super_secret",
        "url": "https://github.com",
        "notes": "",
        "category": "Login",
        "modified": "2026-03-15",
    },
    {
        "id": "2",
        "site": "Google",
        "username": "adam@gmail.com",
        "password": "correcthorsebatterystaple",
        "url": "https://google.com",
        "notes": "Personal account",
        "category": "Login",
        "modified": "2026-03-10",
    },
    {
        "id": "3",
        "site": "University VPN",
        "username": "student123",
        "password": "vpn_pass_9821",
        "url": "",
        "notes": "Connect before 9am",
        "category": "Login",
        "modified": "2026-02-28",
    },
    {
        "id": "4",
        "site": "Visa card",
        "username": "",
        "password": "1234 5678 9012 3456",
        "url": "",
        "notes": "Expires 09/28, CVV 123",
        "category": "Card",
        "modified": "2026-01-01",
    },
]

# ── Public API — these are what the GUI calls ──────────────────────────────────

USE_STUBS = False  # flip to False when real CLI is ready


def vault_exists() -> bool:
    """Return True if a vault file is present on disk."""
    if USE_STUBS:
        return True  # change to False to test the create-vault flow
    return VAULT_PATH.exists()


def create_vault(passphrase: str) -> bool:
    """
    Create a new empty vault encrypted with passphrase.
    Returns True on success.
    """
    if USE_STUBS:
        print("[STUB] create_vault called")
        return True
    VAULT_PATH.parent.mkdir(parents=True, exist_ok=True)
    result = subprocess.run(
        ["passman-cli", "create-vault", "--vault", str(VAULT_PATH)],
        input=passphrase,
        text=True,
        capture_output=True,
    )
    return result.returncode == 0


def unlock(passphrase: str) -> list | None:
    """
    Decrypt the vault with passphrase.
    Returns a list of entry dicts on success, None on wrong passphrase.
    """
    if USE_STUBS:
        print("[STUB] unlock called")
        if passphrase == "wrong":
            return None
        return list(_MOCK_ENTRIES)
    result = subprocess.run(
        ["passman-cli", "unlock", "--vault", str(VAULT_PATH)],
        input=passphrase,
        text=True,
        capture_output=True,
    )
    if result.returncode != 0:
        return None
    return json.loads(result.stdout)


def save(passphrase: str, entries: list) -> bool:
    """
    Encrypt and save the full entries list back to the vault.
    Returns True on success.
    """
    if USE_STUBS:
        print(f"[STUB] save called with {len(entries)} entries")
        return True
    payload = json.dumps(entries)
    result = subprocess.run(
        ["passman-cli", "save", "--vault", str(VAULT_PATH)],
        input=f"{passphrase}\n{payload}",
        text=True,
        capture_output=True,
    )
    return result.returncode == 0


def generate_password(length: int = 20) -> str:
    """Generate a strong random password."""
    if USE_STUBS:
        import secrets
        import string
        alphabet = string.ascii_letters + string.digits + "!@#$%^&*"
        return "".join(secrets.choice(alphabet) for _ in range(length))
    result = subprocess.run(
        ["passman-cli", "gen-password", "--length", str(length)],
        text=True,
        capture_output=True,
    )
    return result.stdout.strip()


def new_entry_id() -> str:
    """Generate a unique ID for a new entry."""
    return str(uuid.uuid4())
