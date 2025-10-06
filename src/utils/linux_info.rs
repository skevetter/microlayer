use anyhow::Result;
use std::fs;

#[derive(Debug, PartialEq)]
pub enum LinuxDistro {
    Ubuntu,
    Debian,
    Alpine,
    Other,
}

/// Detect the Linux distribution
pub fn detect_distro() -> Result<LinuxDistro> {
    if let Ok(contents) = fs::read_to_string("/etc/os-release") {
        if contents.contains("ID=ubuntu") || contents.contains("ID=\"ubuntu\"") {
            return Ok(LinuxDistro::Ubuntu);
        } else if contents.contains("ID=alpine") || contents.contains("ID=\"alpine\"") {
            return Ok(LinuxDistro::Alpine);
        } else if contents.contains("ID=debian")
            || contents.contains("ID=\"debian\"")
            || contents.contains("ID_LIKE=debian")
            || contents.contains("ID_LIKE=\"debian\"")
        {
            return Ok(LinuxDistro::Debian);
        }
    }

    Ok(LinuxDistro::Other)
}

/// Check if the system is Ubuntu
pub fn is_ubuntu() -> bool {
    matches!(detect_distro(), Ok(LinuxDistro::Ubuntu))
}

/// Check if the system is Debian-like
pub fn is_debian_like() -> bool {
    matches!(
        detect_distro(),
        Ok(LinuxDistro::Ubuntu) | Ok(LinuxDistro::Debian)
    )
}

/// Check if the system is Alpine
pub fn is_alpine() -> bool {
    matches!(detect_distro(), Ok(LinuxDistro::Alpine))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_distro_enum() {
        assert_ne!(LinuxDistro::Ubuntu, LinuxDistro::Debian);
        assert_ne!(LinuxDistro::Ubuntu, LinuxDistro::Alpine);
        assert_ne!(LinuxDistro::Debian, LinuxDistro::Alpine);
    }

    #[test]
    fn test_detect_distro_returns_result() {
        let result = detect_distro();
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_ubuntu_returns_bool() {
        let _ = is_ubuntu();
    }

    #[test]
    fn test_is_debian_like_returns_bool() {
        let _ = is_debian_like();
    }

    #[test]
    fn test_is_alpine_returns_bool() {
        let _ = is_alpine();
    }
}
