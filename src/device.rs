//! Hardware abstraction layer. Animations and the engine only ever see
//! the `KeyboardDevice` trait; the concrete sysfs implementation lives
//! here, alongside a `MockDevice` used by unit tests (spec section 48)
//! so the whole test suite runs on machines with no real LED hardware.

use crate::color::Color;
use crate::error::{KbdAnimError, Result};
use crate::sysfs::SysfsFile;
use std::path::PathBuf;

pub const DEFAULT_DEVICE_PATH: &str = "/sys/class/leds/rgb:kbd_backlight";

pub trait KeyboardDevice: Send {
    fn write_color(&mut self, color: Color) -> Result<()>;
    fn write_brightness(&mut self, brightness: u8) -> Result<()>;
    fn read_brightness(&self) -> Result<u8>;
    fn read_color(&self) -> Result<Color>;
    fn device_path(&self) -> String;
}

/// The real hardware backend: reads/writes the sysfs `brightness` and
/// `multi_intensity` files directly (never via a shell subprocess).
/// Caches the last-written values so identical writes are skipped
/// (spec section 41: avoid redundant sysfs writes).
pub struct SysfsDevice {
    base: PathBuf,
    brightness_file: SysfsFile,
    color_file: SysfsFile,
    last_color: Option<Color>,
    last_brightness: Option<u8>,
}

impl SysfsDevice {
    pub fn new(base: PathBuf) -> Self {
        let brightness_file = SysfsFile::new(base.join("brightness"));
        let color_file = SysfsFile::new(base.join("multi_intensity"));
        SysfsDevice {
            base,
            brightness_file,
            color_file,
            last_color: None,
            last_brightness: None,
        }
    }

    pub fn default_device() -> Self {
        SysfsDevice::new(PathBuf::from(DEFAULT_DEVICE_PATH))
    }

    pub fn exists(&self) -> bool {
        self.brightness_file.exists() && self.color_file.exists()
    }

    pub fn brightness_path(&self) -> &std::path::Path {
        self.brightness_file.path()
    }

    pub fn color_path(&self) -> &std::path::Path {
        self.color_file.path()
    }

    /// Forgets any cached last-written values, forcing the next writes
    /// through even if they happen to match. Used after the device
    /// reappears from a hotplug event.
    pub fn invalidate_cache(&mut self) {
        self.last_color = None;
        self.last_brightness = None;
    }
}

impl KeyboardDevice for SysfsDevice {
    fn write_color(&mut self, color: Color) -> Result<()> {
        if self.last_color == Some(color) {
            return Ok(());
        }
        let value = format!("{} {} {}", color.r, color.g, color.b);
        self.color_file.write_string(&value)?;
        self.last_color = Some(color);
        Ok(())
    }

    fn write_brightness(&mut self, brightness: u8) -> Result<()> {
        if self.last_brightness == Some(brightness) {
            return Ok(());
        }
        self.brightness_file.write_string(&brightness.to_string())?;
        self.last_brightness = Some(brightness);
        Ok(())
    }

    fn read_brightness(&self) -> Result<u8> {
        let s = self.brightness_file.read_string()?;
        s.trim()
            .parse::<u32>()
            .map(|v| v.min(255) as u8)
            .map_err(|_| KbdAnimError::Other(format!("could not parse brightness value: {s}")))
    }

    fn read_color(&self) -> Result<Color> {
        let s = self.color_file.read_string()?;
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(KbdAnimError::Other(format!(
                "unexpected multi_intensity format: {s}"
            )));
        }
        let r: u8 = parts[0].parse().unwrap_or(0);
        let g: u8 = parts[1].parse().unwrap_or(0);
        let b: u8 = parts[2].parse().unwrap_or(0);
        Ok(Color::new(r, g, b))
    }

    fn device_path(&self) -> String {
        self.base.display().to_string()
    }
}

/// An in-memory device used by unit tests and by `cargo test` on
/// machines with no real keyboard LED (spec section 48).
#[derive(Debug)]
pub struct MockDevice {
    pub written_colors: Vec<Color>,
    pub written_brightness: Vec<u8>,
    pub current_color: Color,
    pub current_brightness: u8,
    pub fail_writes: bool,
}

impl MockDevice {
    pub fn new() -> Self {
        MockDevice {
            written_colors: Vec::new(),
            written_brightness: Vec::new(),
            current_color: Color::BLACK,
            current_brightness: 0,
            fail_writes: false,
        }
    }
}

impl KeyboardDevice for MockDevice {
    fn write_color(&mut self, color: Color) -> Result<()> {
        if self.fail_writes {
            return Err(KbdAnimError::DeviceWrite("mock device failure".to_string()));
        }
        self.current_color = color;
        self.written_colors.push(color);
        Ok(())
    }

    fn write_brightness(&mut self, brightness: u8) -> Result<()> {
        if self.fail_writes {
            return Err(KbdAnimError::DeviceWrite("mock device failure".to_string()));
        }
        self.current_brightness = brightness;
        self.written_brightness.push(brightness);
        Ok(())
    }

    fn read_brightness(&self) -> Result<u8> {
        Ok(self.current_brightness)
    }

    fn read_color(&self) -> Result<Color> {
        Ok(self.current_color)
    }

    fn device_path(&self) -> String {
        "mock://kbd_backlight".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_device_records_writes() {
        let mut dev = MockDevice::new();
        dev.write_color(Color::new(1, 2, 3)).unwrap();
        dev.write_brightness(100).unwrap();
        assert_eq!(dev.written_colors, vec![Color::new(1, 2, 3)]);
        assert_eq!(dev.written_brightness, vec![100]);
        assert_eq!(dev.read_color().unwrap(), Color::new(1, 2, 3));
        assert_eq!(dev.read_brightness().unwrap(), 100);
    }

    #[test]
    fn mock_device_can_simulate_failures() {
        let mut dev = MockDevice::new();
        dev.fail_writes = true;
        assert!(dev.write_color(Color::BLACK).is_err());
        assert!(dev.write_brightness(0).is_err());
    }
}
