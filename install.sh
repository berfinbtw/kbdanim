#!/usr/bin/env bash
# kbdanim otomatik kurulum betiği
# https://github.com/berfinbtw/kbdanim
set -Eeuo pipefail

REPO_URL="${KBDANIM_REPO_URL:-https://github.com/berfinbtw/kbdanim.git}"
REPO_DIR="${KBDANIM_DIR:-$HOME/kbdanim}"

INSTALL_GUI="auto"
ASSUME_YES=0
SETUP_UDEV=1
SETUP_SERVICE=0
PREFIX="/usr/local"
NEED_RELOGIN=0
SRC_DIR=""

log()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[!]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[x]\033[0m %s\n' "$*" >&2; exit 1; }

usage() {
  cat <<'EOF'
kbdanim otomatik kurulum

Kullanım:
  ./install.sh [seçenekler]

Seçenekler:
  --gui         GUI'yi de kur
  --no-gui      Sadece CLI/daemon kur
  --yes, -y     Soru sorma
  --no-udev     udev kuralını kurma
  --service     systemd kullanıcı servisini etkinleştir
  --prefix YOL  Kurulum öneki (varsayılan: /usr/local)
  -h, --help    Bu yardımı göster

Örnekler:
  ./install.sh --yes
  curl -fsSL https://raw.githubusercontent.com/berfinbtw/kbdanim/main/install.sh | bash
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --gui) INSTALL_GUI=1 ;;
    --no-gui) INSTALL_GUI=0 ;;
    --yes|-y) ASSUME_YES=1 ;;
    --no-udev) SETUP_UDEV=0 ;;
    --service) SETUP_SERVICE=1 ;;
    --prefix)
      [[ $# -ge 2 ]] || die "--prefix için bir yol gerekli."
      PREFIX="$2"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "Bilinmeyen seçenek: $1 (--help yazdır)"
      ;;
  esac
  shift
done

[[ "$PREFIX" == /* ]] || die "--prefix mutlak bir yol olmalı (örn. /usr/local)."
[[ $EUID -ne 0 ]] || die "Bu betiği sudo olmadan, normal kullanıcı olarak çalıştır."

command -v sudo >/dev/null 2>&1 || die "sudo bulunamadı."
# Bazı sudo shim'leri (ör. doas-sudo-shim) -v seçeneğini desteklemez.
sudo true || die "sudo yetkisi alınamadı."

TARGET_USER="$USER"

detect_distro() {
  [[ -r /etc/os-release ]] || die "/etc/os-release okunamadı."
  # shellcheck disable=SC1091
  . /etc/os-release

  DISTRO_ID="${ID:-bilinmiyor}"
  DISTRO_LIKE="${ID_LIKE:-}"

  case "${DISTRO_ID} ${DISTRO_LIKE}" in
    *arch*) PKG=arch ;;
    *debian*|*ubuntu*) PKG=debian ;;
    *fedora*) PKG=fedora ;;
    *suse*) PKG=suse ;;
    *) PKG=unknown ;;
  esac

  log "Dağıtım: ${PRETTY_NAME:-$DISTRO_ID} (${PKG})"
}

detect_gui() {
  [[ "$INSTALL_GUI" == "auto" ]] || return 0

  if [[ -n "${WAYLAND_DISPLAY:-}" || -n "${DISPLAY:-}" || -n "${XDG_CURRENT_DESKTOP:-}" ]]; then
    INSTALL_GUI=1
    log "Masaüstü ortamı algılandı; GUI de kurulacak."
  elif [[ $ASSUME_YES -eq 1 ]]; then
    INSTALL_GUI=0
  elif [[ -t 0 ]]; then
    local answer=""
    read -r -p "GUI de kurulsun mu? [E/h] " answer
    if [[ "$answer" =~ ^[EeYy]?$ ]]; then
      INSTALL_GUI=1
    else
      INSTALL_GUI=0
    fi
  else
    INSTALL_GUI=0
  fi
}

install_packages() {
  log "Gerekli paketler kuruluyor..."
  case "$PKG" in
    arch)
      command -v pacman >/dev/null 2>&1 || die "pacman bulunamadı."
      sudo pacman -S --needed --noconfirm base-devel git curl ca-certificates
      if [[ $INSTALL_GUI -eq 1 ]]; then
        sudo pacman -S --needed --noconfirm \
          libx11 libxcursor libxi libxrandr libxkbcommon wayland mesa
      fi
      ;;
    debian)
      command -v apt-get >/dev/null 2>&1 || die "apt-get bulunamadı."
      sudo apt-get update
      sudo apt-get install -y build-essential git curl ca-certificates
      if [[ $INSTALL_GUI -eq 1 ]]; then
        sudo apt-get install -y \
          libx11-dev libxcursor-dev libxi-dev libxrandr-dev \
          libxkbcommon-dev libwayland-dev libgl1-mesa-dev libegl1-mesa-dev
      fi
      ;;
    fedora)
      command -v dnf >/dev/null 2>&1 || die "dnf bulunamadı."
      sudo dnf install -y gcc gcc-c++ make git curl ca-certificates
      if [[ $INSTALL_GUI -eq 1 ]]; then
        sudo dnf install -y \
          libX11-devel libXcursor-devel libXi-devel libXrandr-devel \
          libxkbcommon-devel wayland-devel mesa-libGL-devel mesa-libEGL-devel
      fi
      ;;
    suse)
      command -v zypper >/dev/null 2>&1 || die "zypper bulunamadı."
      sudo zypper --non-interactive install gcc gcc-c++ make git curl ca-certificates
      if [[ $INSTALL_GUI -eq 1 ]]; then
        sudo zypper --non-interactive install \
          libX11-devel libXcursor-devel libXi-devel libXrandr-devel \
          libxkbcommon-devel wayland-devel Mesa-libGL-devel Mesa-libEGL-devel
      fi
      ;;
    *)
      warn "Bilinmeyen dağıtım: ${PRETTY_NAME:-$DISTRO_ID}"
      warn "git, make, gcc, curl ve Rust'ın kurulu olduğundan emin ol."
      ;;
  esac
}

version_ge() {
  [[ -n "${1:-}" ]] || return 1
  [[ "$(printf '%s\n%s\n' "$2" "$1" | sort -V | head -n 1)" == "$2" ]]
}

ensure_rust() {
  export PATH="$HOME/.cargo/bin:$PATH"

  local min_rust="1.74"
  if [[ $INSTALL_GUI -eq 1 ]]; then
    min_rust="1.76"
  fi

  local need_install=0
  if ! command -v cargo >/dev/null 2>&1; then
    need_install=1
  else
    local rust_version
    rust_version="$(rustc --version 2>/dev/null | awk '{print $2}' || true)"
    if ! version_ge "$rust_version" "$min_rust"; then
      warn "Rust sürümü çok eski: ${rust_version:-bilinmiyor} (gereken: $min_rust)"
      if command -v rustup >/dev/null 2>&1; then
        rustup update stable
      else
        need_install=1
      fi
    fi
  fi

  if [[ $need_install -eq 1 ]]; then
    command -v curl >/dev/null 2>&1 || die "curl bulunamadı."
    log "Rust kuruluyor (rustup)..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    export PATH="$HOME/.cargo/bin:$PATH"
  fi

  command -v cargo >/dev/null 2>&1 || die "cargo bulunamadı. https://rustup.rs adresinden kurup tekrar dene."
  log "Rust: $(rustc --version)"
}

resolve_source() {
  command -v git >/dev/null 2>&1 || die "git bulunamadı."

  local script_dir=""
  if [[ -n "${BASH_SOURCE[0]:-}" && -f "${BASH_SOURCE[0]}" ]]; then
    script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
  fi

  if [[ -n "$script_dir" && -f "$script_dir/Cargo.toml" && -f "$script_dir/Makefile" ]]; then
    SRC_DIR="$script_dir"
    log "Kaynak dizini: $SRC_DIR"
    return 0
  fi

  SRC_DIR="$REPO_DIR"
  if [[ -d "$SRC_DIR/.git" ]]; then
    log "Mevcut depo güncelleniyor: $SRC_DIR"
    git -C "$SRC_DIR" pull --ff-only || warn "git pull başarısız; mevcut kaynakla devam ediliyor."
  elif [[ -f "$SRC_DIR/Cargo.toml" ]]; then
    log "Mevcut kaynak dizini kullanılıyor: $SRC_DIR"
  elif [[ -e "$SRC_DIR" ]]; then
    die "$SRC_DIR var ama geçerli bir kbdanim kaynağı değil. KBDANIM_DIR ile başka bir yol ver."
  else
    log "Depo klonlanıyor: $REPO_URL"
    git clone --depth 1 "$REPO_URL" "$SRC_DIR"
  fi
}

install_cli() {
  command -v make >/dev/null 2>&1 || die "make bulunamadı."
  cd "$SRC_DIR"

  log "CLI/daemon derleniyor..."
  make

  log "CLI/daemon kuruluyor: $PREFIX"
  sudo install -Dm755 target/release/kbdanim "$PREFIX/bin/kbdanim"
  sudo install -Dm644 man/kbdanim.1 "$PREFIX/share/man/man1/kbdanim.1"
  sudo install -Dm644 completions/kbdanim.bash /etc/bash_completion.d/kbdanim
  sudo install -Dm644 completions/kbdanim.zsh "$PREFIX/share/zsh/site-functions/_kbdanim"
  sudo install -Dm644 completions/kbdanim.fish "$PREFIX/share/fish/vendor_completions.d/kbdanim.fish"
}

install_gui() {
  cd "$SRC_DIR"

  log "GUI derleniyor..."
  make build-gui

  log "GUI kuruluyor..."
  sudo install -Dm755 target/release/kbdanim-gui "$PREFIX/bin/kbdanim-gui"
  sudo install -Dm644 assets/kbdanim.desktop "$PREFIX/share/applications/kbdanim.desktop"
  sudo install -Dm644 assets/kbdanim.svg "$PREFIX/share/icons/hicolor/scalable/apps/kbdanim.svg"

  if command -v update-desktop-database >/dev/null 2>&1; then
    sudo update-desktop-database "$PREFIX/share/applications" >/dev/null 2>&1 || true
  fi
  if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    sudo gtk-update-icon-cache -f "$PREFIX/share/icons/hicolor" >/dev/null 2>&1 || true
  fi
}

setup_udev() {
  local rule="$SRC_DIR/udev/60-kbdanim.rules"
  if [[ ! -f "$rule" ]]; then
    warn "udev kuralı bulunamadı, atlanıyor."
    return 0
  fi

  log "udev kuralı kuruluyor..."
  sudo groupadd -f kbdanim

  if ! id -nG "$TARGET_USER" | tr ' ' '\n' | grep -qx "kbdanim"; then
    sudo usermod -aG kbdanim "$TARGET_USER"
    NEED_RELOGIN=1
  fi

  sudo install -Dm644 "$rule" /etc/udev/rules.d/60-kbdanim.rules

  if command -v udevadm >/dev/null 2>&1; then
    sudo udevadm control --reload-rules || warn "udev kuralları yeniden yüklenemedi."
    sudo udevadm trigger --subsystem-match=leds || true
  else
    warn "udevadm bulunamadı; değişiklik için sistemi yeniden başlat."
  fi
}

setup_service() {
  local unit="$SRC_DIR/systemd/kbdanim.service"
  if [[ ! -f "$unit" ]]; then
    warn "systemd servis dosyası bulunamadı, atlanıyor."
    return 0
  fi

  log "systemd kullanıcı servisi etkinleştiriliyor..."
  mkdir -p "$HOME/.config/systemd/user"
  sed "s|/usr/local/bin/kbdanim|$PREFIX/bin/kbdanim|g" "$unit" \
    > "$HOME/.config/systemd/user/kbdanim.service"
  chmod 644 "$HOME/.config/systemd/user/kbdanim.service"

  if systemctl --user daemon-reload 2>/dev/null; then
    systemctl --user enable kbdanim || warn "Servis etkinleştirilemedi."
    if [[ $NEED_RELOGIN -eq 0 ]]; then
      systemctl --user restart kbdanim || warn "Servis başlatılamadı. 'systemctl --user status kbdanim' ile kontrol et."
    else
      warn "Grup üyeliği için oturumu yeniledikten sonra: systemctl --user start kbdanim"
    fi
  else
    warn "systemd kullanıcı oturumu bulunamadı; servisi elle etkinleştir."
  fi
}

detect_distro
detect_gui
install_packages
ensure_rust
resolve_source

if [[ ! -e /sys/class/leds/rgb:kbd_backlight ]]; then
  warn "Varsayılan klavye LED cihazı bulunamadı: /sys/class/leds/rgb:kbd_backlight"
  warn "Farklı bir cihaz adı kullanıyorsan --device ile belirtmen gerekecek."
fi

install_cli
if [[ $INSTALL_GUI -eq 1 ]]; then
  install_gui
fi
if [[ $SETUP_UDEV -eq 1 ]]; then
  setup_udev
fi
if [[ $SETUP_SERVICE -eq 1 ]]; then
  setup_service
fi

hash -r 2>/dev/null || true

log "Kurulum tamamlandı."
echo
echo "Kontroller:"
echo "  kbdanim doctor"
echo "  kbdanim list"
echo "  kbdanim run --mode rainbow"
if [[ $INSTALL_GUI -eq 1 ]]; then
  echo "  kbdanim-gui"
fi
echo
if [[ $NEED_RELOGIN -eq 1 ]]; then
  warn "udev grubu için oturumu kapatıp yeniden açmalısın."
fi
