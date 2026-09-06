#!/usr/bin/env bash
# Install + start Crosslink zebrad as a systemd --user service (background, Restart=always).
set -euo pipefail

ZEBRAD_GUI="/home/lowo/projects/crosslink-monolith-v13/zebra-crosslink/target/release/zebrad"
ZEBRAD_HEADLESS="/home/lowo/zebra-crosslink/zebrad-headless"
# Prefer headless binary (no viz_gui) for background service.
if [[ -x "$ZEBRAD_HEADLESS" ]]; then
  ZEBRAD="$ZEBRAD_HEADLESS"
elif [[ -x "$ZEBRAD_GUI" ]]; then
  ZEBRAD="$ZEBRAD_GUI"
  echo "WARNING: using GUI-enabled zebrad; systemd may crash without a display."
  echo "  Build headless with: cargo build --release -p zebrad  (no -F viz_gui)"
else
  echo "Missing zebrad binary" >&2
  exit 1
fi
CONF_DIR="/home/lowo/zebra-crosslink"
CONF="$CONF_DIR/zebrad.toml"
CACHE_DIR="$CONF_DIR/cache"
UNIT_DIR="$HOME/.config/systemd/user"
UNIT="$UNIT_DIR/zebrad-crosslink.service"
RPC_PORT=18232
P2P_PORT=18233

mkdir -p "$CONF_DIR" "$CACHE_DIR" "$UNIT_DIR"

# Ensure config exists (generate + patch if missing).
if [[ ! -f "$CONF" ]]; then
  echo "Generating Crosslink config..."
  "$ZEBRAD" generate -o "$CONF"
fi

python3 - <<'PY'
from pathlib import Path
import re

conf = Path("/home/lowo/zebra-crosslink/zebrad.toml")
text = conf.read_text()

def set_in_section(src: str, section: str, key: str, value: str) -> str:
    sec_re = re.compile(rf"^\[{re.escape(section)}\]\s*$", re.M)
    m = sec_re.search(src)
    if not m:
        return src.rstrip() + f"\n\n[{section}]\n{key} = {value}\n"
    start = m.end()
    next_sec = re.search(r"^\[", src[start:], re.M)
    end = start + next_sec.start() if next_sec else len(src)
    body = src[start:end]
    key_re = re.compile(rf"(?m)^(\s*{re.escape(key)}\s*=\s*).*$")
    if key_re.search(body):
        body = key_re.sub(rf"\1{value}", body, count=1)
    else:
        body = f"\n{key} = {value}\n" + body
    return src[:start] + body + src[end:]

text = set_in_section(text, "state", "cache_dir", '"/home/lowo/zebra-crosslink/cache"')
text = set_in_section(text, "rpc", "cookie_dir", '"/home/lowo/zebra-crosslink/cache"')
text = set_in_section(text, "rpc", "listen_addr", '"0.0.0.0:18232"')
text = set_in_section(text, "rpc", "enable_cookie_auth", "false")
text = set_in_section(text, "network", "listen_addr", '"[::]:18233"')
conf.write_text(text)
print(f"Config ready: {conf}")
PY

# Stop any leftover one-shot nohup instance from earlier setup.
if [[ -f "$CONF_DIR/zebrad.pid" ]]; then
  old="$(cat "$CONF_DIR/zebrad.pid" || true)"
  if [[ -n "${old:-}" ]] && kill -0 "$old" 2>/dev/null; then
    echo "Stopping leftover pid $old"
    kill "$old" || true
    sleep 1
  fi
  rm -f "$CONF_DIR/zebrad.pid"
fi
pkill -f 'zebra-crosslink/zebrad.toml' 2>/dev/null || true
sleep 1

cat >"$UNIT" <<EOF
[Unit]
Description=Nozy Crosslink Season zebrad (feature-net)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=/home/lowo/zebra-crosslink
ExecStart=${ZEBRAD} -c ${CONF} start
Restart=always
RestartSec=5
# Keep logs out of journal spam; also write a file like mainnet.
StandardOutput=append:/home/lowo/zebra-crosslink/zebrad-crosslink.log
StandardError=append:/home/lowo/zebra-crosslink/zebrad-crosslink.log
# GUI/audio probes not needed for headless binary.
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=default.target
EOF

echo "Wrote $UNIT"

# Linger keeps user services after logout. Skip if it needs a password (non-interactive).
if command -v loginctl >/dev/null 2>&1; then
  if ! loginctl show-user "$USER" -p Linger 2>/dev/null | grep -q 'Linger=yes'; then
    if loginctl enable-linger "$USER" 2>/dev/null; then
      echo "Linger enabled for $USER"
    else
      echo "Note: could not enable linger without sudo. Service still runs while WSL is up."
      echo "  Optional later: sudo loginctl enable-linger $USER"
    fi
  fi
fi

systemctl --user daemon-reload
systemctl --user enable zebrad-crosslink.service
systemctl --user restart zebrad-crosslink.service
sleep 3
systemctl --user --no-pager status zebrad-crosslink.service || true

echo "---- RPC probe ----"
curl -sS --max-time 8 \
  --data '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}' \
  -H 'content-type: application/json' \
  "http://127.0.0.1:${RPC_PORT}" || echo RPC_NOT_READY_YET
echo
echo "Service: systemctl --user status zebrad-crosslink"
echo "Logs:    tail -f /home/lowo/zebra-crosslink/zebrad-crosslink.log"
echo "Stop:    systemctl --user stop zebrad-crosslink"
echo "RPC:     http://\$(hostname -I | awk '{print \$1}'):${RPC_PORT}  (P2P :${P2P_PORT})"
