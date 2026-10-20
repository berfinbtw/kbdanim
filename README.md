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

## CLI ve daemon kurulumu

Projeyi indir:

```bash
git clone https://github.com/berfinbtw/kbdanim.git
cd kbdanim
```

Zip ile indirdiysen:

```bash
unzip kbdanim.zip
cd kbdanim
```

Kur:

```bash
sudo make install
```

> Rust'ı rustup ile kendi kullanıcına kurduysan `sudo` ortamında `cargo`
> bulunamayabilir. Bu durumda:
>
> ```bash
> make
> sudo env "PATH=$PATH" make install
> ```

Kurulumu doğrula:

```bash
kbdanim doctor
```

## GUI kurulumu (isteğe bağlı)

Önce CLI'yi kurmuş olman gerekir.

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

GUI'yi kur:

```bash
sudo make install-gui
```

Kurulumdan sonra uygulama menüsünde **kbdanim** olarak görünür veya
terminalden `kbdanim-gui` ile açılır.

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
