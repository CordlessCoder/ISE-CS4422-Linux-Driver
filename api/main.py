from flask import Flask, request, jsonify, send_from_directory
from functools import wraps
from pathlib import Path
from cli_wrapper import CLIWrapper
from users import UserManager

app = Flask(__name__, static_folder='static', static_url_path='')
users = UserManager()


def require_auth(f):
    """Decorator to require valid auth token."""
    @wraps(f)
    def decorated_function(*args, **kwargs):
        token = request.headers.get('Authorization')

        if not token or not token.startswith('Bearer '):
            return jsonify({"status": "error", "error": "Missing or invalid token"}), 401

        token = token[7:]  # Remove 'Bearer ' prefix
        valid, username = users.verify_token(token)

        if not valid:
            return jsonify({"status": "error", "error": "Invalid or expired token"}), 401

        return f(username=username, *args, **kwargs)

    return decorated_function


def get_user_vault_dir(username: str) -> Path:
    """Get vault directory for user."""
    vault_dir = Path("./vaults") / username
    vault_dir.mkdir(parents=True, exist_ok=True)
    return vault_dir


@app.route("/register", methods=["POST"])
def register():
    data = request.get_json()

    if not data or "username" not in data or "password" not in data:
        return jsonify({"status": "error", "error": "need username and password"}), 400

    username = data["username"].strip()
    password = data["password"]

    result = users.register(username, password)

    if result["success"]:
        return jsonify({"status": "ok", "message": result["message"]}), 201

    return jsonify({"status": "error", "error": result["error"]}), 400


@app.route("/login", methods=["POST"])
def login():
    data = request.get_json()

    if not data or "username" not in data or "password" not in data:
        return jsonify({"status": "error", "error": "need username and password"}), 400

    username = data["username"].strip()
    password = data["password"]

    result = users.login(username, password)

    if result["success"]:
        return jsonify({
            "status": "ok",
            "token": result["token"],
            "username": result["username"]
        }), 200

    return jsonify({"status": "error", "error": result["error"]}), 401


@app.route("/logout", methods=["POST"])
@require_auth
def logout(username):
    token = request.headers.get('Authorization')[7:]
    users.logout(token)
    return jsonify({"status": "ok"}), 200


@app.route("/create", methods=["POST"])
@require_auth
def create_vault(username):
    data = request.get_json()

    if not data or "name" not in data or "password" not in data:
        return jsonify({"error": "need name and password"}), 400

    vault_name = data["name"].strip()
    password = data["password"]

    if not vault_name:
        return jsonify({"error": "vault name empty"}), 400

    vault_dir = get_user_vault_dir(username)
    cli = CLIWrapper(vaults_dir=str(vault_dir))
    result = cli.create_vault(vault_name, password)

    if result["success"]:
        return jsonify({"status": "ok", "message": f"created '{vault_name}'"}), 201

    return jsonify({"status": "error", "error": result["error"]}), 400


@app.route("/unlock", methods=["POST"])
@require_auth
def unlock_vault(username):
    data = request.get_json()

    if not data or "name" not in data or "password" not in data:
        return jsonify({"error": "need name and password"}), 400

    vault_name = data["name"].strip()
    password = data["password"]

    if not vault_name:
        return jsonify({"error": "vault name empty"}), 400

    vault_dir = get_user_vault_dir(username)
    cli = CLIWrapper(vaults_dir=str(vault_dir))
    result = cli.unlock_vault(vault_name, password)

    if result["success"]:
        return jsonify({"status": "ok", "data": result["data"]}), 200

    return jsonify({"status": "error", "error": result["error"]}), 400


@app.route("/save", methods=["POST"])
@require_auth
def save_vault(username):
    data = request.get_json()

    if not data or "name" not in data or "password" not in data or "data" not in data:
        return jsonify({"error": "need name, password, and data"}), 400

    vault_name = data["name"].strip()
    password = data["password"]
    vault_data = data["data"]

    if not vault_name:
        return jsonify({"error": "vault name empty"}), 400

    vault_dir = get_user_vault_dir(username)
    cli = CLIWrapper(vaults_dir=str(vault_dir))
    result = cli.save_vault(vault_name, password, vault_data)

    if result["success"]:
        return jsonify({"status": "ok", "message": f"saved '{vault_name}'"}), 200

    return jsonify({"status": "error", "error": result["error"]}), 400


@app.route("/health", methods=["GET"])
def health():
    return jsonify({"status": "ok"}), 200


@app.route("/", methods=["GET"])
def index():
    return send_from_directory('static', 'index.html')


if __name__ == "__main__":
    app.run(debug=True, host="0.0.0.0", port=5000)
