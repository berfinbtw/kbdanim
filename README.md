# kbdanim

Linux'ta tek bölgeli klavye arka ışığını yöneten, Rust ile yazılmış
animasyon motoru. Terminal için `kbdanim`, isteğe bağlı masaüstü arayüzü
için `kbdanim-gui` içerir.

## Gereksinimler

- Linux
- Klavye arka ışığı: `/sys/class/leds/rgb:kbd_backlight`
- Rust 1.74 veya üzeri (GUI için 1.76+)
- GUI için X11/Wayland geliştirme kütüphaneleri

Cihazını kontrol et:

```bash
ls /sys/class/leds/ | grep kbd_backlight
```

Farklı bir cihaz adı kullanıyorsan `--device /sys/class/leds/ADINIZ`
veya `~/.config/kbdanim/config.toml` içindeki `device` ayarını kullan.

## Tek komutla kurulum

```bash
curl -fsSL https://raw.githubusercontent.com/berfinbtw/kbdanim/main/install.sh | bash
```

Kurulum scripti:

- Dağıtımı algılar (Arch, Debian/Ubuntu, Fedora, openSUSE)
- Gerekli paketleri ve Rust'ı kurar
- Projeyi `~/kbdanim` altına klonlar
- CLI/daemon'u derler ve kurar
- Masaüstü ortamı algılarsa GUI'yi de kurar
- udev kuralını kurar

> Scripti `sudo` ile çalıştırma; sudo gerektiğinde kendisi kullanır.

Sadece CLI/daemon için:

```bash
curl -fsSL https://raw.githubusercontent.com/berfinbtw/kbdanim/main/install.sh | bash -s -- --no-gui
```

### Kurulum seçenekleri

```bash
./install.sh --no-gui      # GUI'siz kur
./install.sh --gui         # GUI'yi zorla kur
./install.sh --no-udev     # udev kuralını kurma
./install.sh --service     # systemd kullanıcı servisini etkinleştir
./install.sh --yes         # soru sorma
./install.sh --prefix YOL  # farklı kurulum öneki (varsayılan: /usr/local)
./install.sh --help
```

## Kurulum scripti ile

```bash
git clone https://github.com/berfinbtw/kbdanim.git
cd kbdanim
chmod +x install.sh
./install.sh
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
make build-gui
sudo make install-gui
```

GUI bağımlılıkları:

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

## sudo olmadan kullanım

Kurulum scripti udev kuralını varsayılan olarak kurar. Kuralın etkin
olması için oturumu kapatıp yeniden aç:

```bash
kbdanim doctor
```

Kuralı elle kurmak istersen:

```bash
sudo groupadd -f kbdanim
sudo usermod -aG kbdanim "$USER"
sudo install -Dm644 udev/60-kbdanim.rules /etc/udev/rules.d/60-kbdanim.rules
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=leds
```

Cihaza zaten `leds` grubu üzerinden erişimin varsa ve mevcut erişimi
bozmak istemiyorsan kurulumu `--no-udev` ile çalıştır.

## Arka planda çalıştırma

Kurulum scripti ile:

```bash
./install.sh --service
```

Elle:

```bash
mkdir -p ~/.config/systemd/user
cp systemd/kbdanim.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now kbdanim
```

Kontrol:

```bash
systemctl --user status kbdanim
kbdanim status
```

## Kullanım

```bash
kbdanim list
kbdanim run --mode rainbow
kbdanim run --mode fire --speed 1.5
kbdanim static --color "#00aaff"
kbdanim off

kbdanim preset run gaming
kbdanim playlist run evening
kbdanim-gui
```

## Sorun giderme

- **`Permission denied`**: udev kuralını kurup oturumu yeniden aç.
- **`cargo: command not found`**: `sudo env "PATH=$PATH" make install`
  kullan veya Rust'ı sistem geneline kur.
- **Uygulama menüsünde görünmüyor**:
  `sudo update-desktop-database /usr/local/share/applications`
- **GUI açılmıyor**: X11/Wayland geliştirme kütüphanelerinin kurulu
  olduğundan emin ol.
- **Cihaz bulunamıyor**: `ls /sys/class/leds/` çıktısını kontrol et ve
  `--device` ile doğru yolu ver.

## Lisans

MIT
