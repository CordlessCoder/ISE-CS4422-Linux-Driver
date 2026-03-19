# Passman GUI

GTK4 graphical interface for the passman password manager.

## Prerequisites

GTK4 must be installed at the system level first.

**Ubuntu / Linux Mint:**
```bash
sudo apt install python3-gi python3-gi-cairo gir1.2-gtk-4.0
```

**Arch / CachyOS / Manjaro:**
```bash
sudo pacman -S python-gobject gtk4
```

## Running
```bash
cd gui/
python3 main.py
```

## Running with uv
```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
cd gui/
uv sync
uv run python3 main.py
```