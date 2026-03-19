from flask import Flask, request, jsonify, send_from_directory
from cli_wrapper import CLIWrapper

app = Flask(__name__, static_folder='static', static_url_path='')
cli = CLIWrapper(vaults_dir="./vaults")


@app.route("/create", methods=["POST"])
def create_vault():
    data = request.get_json()

    if not data or "name" not in data or "password" not in data:
        return jsonify({"error": "need name and password"}), 400

    vault_name = data["name"].strip()
    password = data["password"]

    if not vault_name:
        return jsonify({"error": "vault name empty"}), 400

    result = cli.create_vault(vault_name, password)

    if result["success"]:
        return jsonify({"status": "ok", "message": f"created '{vault_name}'"}), 201

    return jsonify({"status": "error", "error": result["error"]}), 400


@app.route("/unlock", methods=["POST"])
def unlock_vault():
    data = request.get_json()

    if not data or "name" not in data or "password" not in data:
        return jsonify({"error": "need name and password"}), 400

    vault_name = data["name"].strip()
    password = data["password"]

    if not vault_name:
        return jsonify({"error": "vault name empty"}), 400

    result = cli.unlock_vault(vault_name, password)

    if result["success"]:
        return jsonify({"status": "ok", "data": result["data"]}), 200

    return jsonify({"status": "error", "error": result["error"]}), 400


@app.route("/save", methods=["POST"])
def save_vault():
    data = request.get_json()

    if not data or "name" not in data or "password" not in data or "data" not in data:
        return jsonify({"error": "need name, password, and data"}), 400

    vault_name = data["name"].strip()
    password = data["password"]
    vault_data = data["data"]

    if not vault_name:
        return jsonify({"error": "vault name empty"}), 400

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
