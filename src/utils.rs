pub mod analytics;
pub mod command;
pub mod linux_info;
pub mod locking;

#[cfg(test)]
mod tests {
    #[test]
    fn test_command_module_exists() {
        use super::command;
        let _ = command::execute_status("true");
    }

    #[test]
    fn test_linux_info_module_exists() {
        use super::linux_info;
        let _ = linux_info::detect_distro();
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
