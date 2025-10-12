pub mod analytics;
pub mod filesystem;
pub mod locking;
pub mod logging;
pub mod os;
pub mod pkgx;

#[cfg(test)]
mod tests {

    #[test]
    fn test_linux_info_module_exists() {
        use super::os;
        let _ = os::detect_distro();
    }

    #[test]
    fn test_analytics_module_exists() {
        use super::analytics;
        let _ = analytics::track_event("test", None);
    }

    #[test]
    fn test_locking_module_exists() {
        use super::locking;
        let _ = locking::acquire_lock();
    }
}
