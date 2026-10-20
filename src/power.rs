//! Detects whether the system is currently running on AC power or
//! battery, used for automatic profile switching (spec sections 23-24).
//! Reads `/sys/class/power_supply/*/type` and `.../online`; never
//! panics even if the directory or its entries are missing (e.g. in a
//! container, or on a desktop with no battery).

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    Ac,
    Battery,
    Unknown,
}

pub fn detect() -> PowerState {
    detect_at(Path::new("/sys/class/power_supply"))
}

fn detect_at(base: &Path) -> PowerState {
    let entries = match std::fs::read_dir(base) {
        Ok(e) => e,
        Err(_) => return PowerState::Unknown,
    };

    let mut saw_battery = false;
    for entry in entries.flatten() {
        let path = entry.path();
        let kind = std::fs::read_to_string(path.join("type")).unwrap_or_default();
        let kind = kind.trim();
        if kind == "Mains" || kind == "USB" {
            let online = std::fs::read_to_string(path.join("online")).unwrap_or_default();
            if online.trim() == "1" {
                return PowerState::Ac;
            }
        } else if kind == "Battery" {
            saw_battery = true;
        }
    }

    if saw_battery {
        PowerState::Battery
    } else {
        PowerState::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_power_supply_dir_is_unknown() {
        assert_eq!(detect_at(Path::new("/nonexistent/power_supply_test")), PowerState::Unknown);
    }
}
