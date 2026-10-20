//! Direct filesystem access to the LED's sysfs attribute files. This
//! deliberately never shells out to `tee`/`echo`; every write goes
//! through `std::fs` directly, as required by the spec.

use crate::error::{KbdAnimError, Result};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub struct SysfsFile {
    path: PathBuf,
}

impl SysfsFile {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        SysfsFile { path: path.into() }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn read_string(&self) -> Result<String> {
        let mut f = std::fs::File::open(&self.path).map_err(|e| map_io_err(e, &self.path))?;
        let mut s = String::new();
        f.read_to_string(&mut s)
            .map_err(|e| map_io_err(e, &self.path))?;
        Ok(s.trim().to_string())
    }

    pub fn write_string(&self, value: &str) -> Result<()> {
        let mut f = OpenOptions::new()
            .write(true)
            .open(&self.path)
            .map_err(|e| map_io_err(e, &self.path))?;
        f.write_all(value.as_bytes())
            .map_err(|e| KbdAnimError::DeviceWrite(format!("{}: {}", self.path.display(), e)))?;
        Ok(())
    }
}

fn map_io_err(e: std::io::Error, path: &Path) -> KbdAnimError {
    match e.kind() {
        std::io::ErrorKind::NotFound => {
            KbdAnimError::DeviceNotFound(path.display().to_string())
        }
        std::io::ErrorKind::PermissionDenied => {
            KbdAnimError::PermissionDenied(path.display().to_string())
        }
        _ => KbdAnimError::Io(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_round_trip_on_temp_file() {
        let mut path = std::env::temp_dir();
        path.push(format!("kbdanim-sysfs-test-{}", std::process::id()));
        std::fs::File::create(&path).unwrap();
        let sf = SysfsFile::new(&path);
        assert!(sf.exists());
        sf.write_string("42").unwrap();
        assert_eq!(sf.read_string().unwrap(), "42");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_file_reports_device_not_found() {
        let sf = SysfsFile::new("/nonexistent/kbdanim/path/for/tests");
        match sf.read_string() {
            Err(KbdAnimError::DeviceNotFound(_)) => {}
            other => panic!("expected DeviceNotFound, got {other:?}"),
        }
    }
}
