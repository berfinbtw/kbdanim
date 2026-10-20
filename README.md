# kbdanim

Linux klavye arka ışığı animasyon motoru için kurulum talimatları.

## Gereksinimler

- Linux
- Rust 1.74 veya üzeri → https://rustup.rs
- Klavye arka ışığı: `/sys/class/leds/rgb:kbd_backlight`

Cihazı kontrol et:

```bash
ls /sys/class/leds/ | grep kbd_backlight
```

## Tek komutla kurulum

```bash
curl -fsSL https://raw.githubusercontent.com/berfinbtw/kbdanim/main/install.sh | bash
```

Script dağıtımı algılar, gerekli paketleri ve Rust'ı kurar, projeyi
`~/kbdanim` altına klonlar, CLI/daemon'u derler ve kurar. Masaüstü
ortamı varsa GUI'yi de kurar.

Sadece CLI/daemon için:

```bash
curl -fsSL https://raw.githubusercontent.com/berfinbtw/kbdanim/main/install.sh | bash -s -- --no-gui
```

## Kurulum scripti ile

```bash
git clone https://github.com/berfinbtw/kbdanim.git
cd kbdanim
./install.sh
```

Seçenekler:

```bash
./install.sh --no-gui      # GUI'siz kur
./install.sh --no-udev     # udev kuralını kurma
./install.sh --service     # systemd kullanıcı servisini etkinleştir
./install.sh --yes         # soru sorma
./install.sh --help
```

## Elle kurulum

```bash
git clone https://github.com/berfinbtw/kbdanim.git
cd kbdanim
sudo make install
```

> Rust'ı rustup ile kendi kullanıcına kurduysan `sudo` ortamında `cargo`
> bulunamayabilir. Bu durumda:
>
> ```bash
> make
> sudo env "PATH=$PATH" make install
> ```

GUI'yi de kurmak için:

```bash
sudo make install-gui
```

GUI için ek X11/Wayland kütüphaneleri gerekir:

**Arch / CachyOS**

```bash
sudo pacman -S --needed libx11 libxcursor libxi libxrandr libxkbcommon wayland mesa
```

**Debian / Ubuntu**

```bash
sudo apt install libx11-dev libxcursor-dev libxi-dev libxrandr-dev \
  libxkbcommon-dev libwayland-dev libgl1-mesa-dev
```

**Fedora**

```bash
sudo dnf install libX11-devel libXcursor-devel libXi-devel libXrandr-devel \
  libxkbcommon-devel wayland-devel mesa-libGL-devel
```

## Kurulumu doğrula

```bash
kbdanim doctor
kbdanim list
kbdanim status
kbdanim-gui
```

## sudo olmadan kullanım (isteğe bağlı)

```bash
sudo groupadd -f kbdanim
sudo usermod -aG kbdanim "$USER"
sudo install -Dm644 udev/60-kbdanim.rules /etc/udev/rules.d/60-kbdanim.rules
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=leds
```

Oturumu kapatıp yeniden açtıktan sonra kontrol et:

```bash
kbdanim doctor
```

## Arka planda çalıştırma (isteğe bağlı)

```bash
mkdir -p ~/.config/systemd/user
cp systemd/kbdanim.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now kbdanim
```
