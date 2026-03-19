import subprocess
from pathlib import Path


class CLIWrapper:
    def __init__(self, binary_path=None, vaults_dir="./vaults"):
        self.binary_path = binary_path or "passman_cli"
        self.vaults_dir = Path(vaults_dir)
        self.vaults_dir.mkdir(exist_ok=True)

    def _get_vault_path(self, vault_name):
        return self.vaults_dir / f"{vault_name}.vault"

    def create_vault(self, vault_name, password):
        vault_path = self._get_vault_path(vault_name)

        if vault_path.exists():
            return {"success": False, "error": f"Vault '{vault_name}' already exists"}

        try:
            cmd = [self.binary_path, "create-vault", "--vault", str(vault_path)]
            proc = subprocess.run(
                cmd,
                input=password.encode() + b"\n",
                capture_output=True,
                timeout=30
            )

            if proc.returncode == 0:
                return {"success": True}
            return {"success": False, "error": proc.stderr.decode().strip()}
        except FileNotFoundError:
            return {"success": False, "error": f"passman_rs not found at {self.binary_path}"}
        except subprocess.TimeoutExpired:
            return {"success": False, "error": "Timeout"}
        except Exception as e:
            return {"success": False, "error": str(e)}

    def unlock_vault(self, vault_name, password):
        vault_path = self._get_vault_path(vault_name)

        if not vault_path.exists():
            return {"success": False, "error": f"Vault '{vault_name}' not found"}

        try:
            cmd = [self.binary_path, "unlock", "--vault", str(vault_path)]
            proc = subprocess.run(
                cmd,
                input=password.encode() + b"\n",
                capture_output=True,
                timeout=30
            )

            if proc.returncode == 0:
                return {"success": True, "data": proc.stdout.decode()}
            return {"success": False, "error": proc.stderr.decode().strip()}
        except subprocess.TimeoutExpired:
            return {"success": False, "error": "Timeout"}
        except Exception as e:
            return {"success": False, "error": str(e)}

    def save_vault(self, vault_name, password, data):
        vault_path = self._get_vault_path(vault_name)

        if not vault_path.exists():
            return {"success": False, "error": f"Vault '{vault_name}' not found"}

        try:
            cmd = [self.binary_path, "save", "--vault", str(vault_path)]
            input_data = password.encode() + b"\n" + data.encode()

            proc = subprocess.run(
                cmd,
                input=input_data,
                capture_output=True,
                timeout=30
            )

            if proc.returncode == 0:
                return {"success": True}
            return {"success": False, "error": proc.stderr.decode().strip()}
        except subprocess.TimeoutExpired:
            return {"success": False, "error": "Timeout"}
        except Exception as e:
            return {"success": False, "error": str(e)}
