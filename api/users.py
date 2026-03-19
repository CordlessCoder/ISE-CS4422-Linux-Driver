"""
User authentication and management.
Stores users in a JSON file with hashed passwords.
"""

import json
import os
from pathlib import Path
from hashlib import pbkdf2_hmac
import secrets
import string


class UserManager:
    def __init__(self, users_file="./users.json"):
        self.users_file = Path(users_file)
        self.users_file.parent.mkdir(parents=True, exist_ok=True)
        self._ensure_file()

    def _ensure_file(self):
        """Create empty users file if it doesn't exist."""
        if not self.users_file.exists():
            self.users_file.write_text(json.dumps({}))

    def _hash_password(self, password: str, salt: str = None) -> tuple:
        """Hash password with PBKDF2. Returns (hash, salt)."""
        if salt is None:
            salt = secrets.token_hex(16)

        hashed = pbkdf2_hmac(
            'sha256',
            password.encode(),
            salt.encode(),
            100000  # iterations
        ).hex()

        return hashed, salt

    def _verify_password(self, password: str, stored_hash: str, salt: str) -> bool:
        """Verify password against stored hash."""
        hashed, _ = self._hash_password(password, salt)
        return hashed == stored_hash

    def register(self, username: str, password: str) -> dict:
        """Register new user. Returns {success, message}."""
        username = username.strip()

        if not username or len(username) < 3:
            return {"success": False, "error": "Username must be at least 3 characters"}

        if not password or len(password) < 8:
            return {"success": False, "error": "Password must be at least 8 characters"}

        users = json.loads(self.users_file.read_text())

        if username in users:
            return {"success": False, "error": f"User '{username}' already exists"}

        hashed, salt = self._hash_password(password)

        users[username] = {
            "password_hash": hashed,
            "salt": salt
        }

        self.users_file.write_text(json.dumps(users, indent=2))

        return {"success": True, "message": f"User '{username}' created"}

    def login(self, username: str, password: str) -> dict:
        """Authenticate user. Returns {success, message}."""
        username = username.strip()

        users = json.loads(self.users_file.read_text())

        if username not in users:
            return {"success": False, "error": "Invalid username or password"}

        user = users[username]

        if not self._verify_password(password, user["password_hash"], user["salt"]):
            return {"success": False, "error": "Invalid username or password"}

        # Generate session token
        token = secrets.token_urlsafe(32)

        # Store token (in production, use Redis or database)
        sessions = self._load_sessions()
        sessions[token] = username
        self._save_sessions(sessions)

        return {"success": True, "token": token, "username": username}

    def verify_token(self, token: str) -> tuple:
        """Verify session token. Returns (valid, username)."""
        sessions = self._load_sessions()

        if token not in sessions:
            return False, None

        return True, sessions[token]

    def logout(self, token: str) -> dict:
        """Logout user by invalidating token."""
        sessions = self._load_sessions()

        if token in sessions:
            del sessions[token]
            self._save_sessions(sessions)

        return {"success": True}

    def _load_sessions(self) -> dict:
        """Load active sessions from file."""
        sessions_file = Path("./sessions.json")
        if not sessions_file.exists():
            return {}
        return json.loads(sessions_file.read_text())

    def _save_sessions(self, sessions: dict):
        """Save active sessions to file."""
        Path("./sessions.json").write_text(json.dumps(sessions))
