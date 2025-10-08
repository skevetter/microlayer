pub mod command;
pub mod linux_info;

#[cfg(test)]
mod tests {
    #[test]
    fn test_command_module_exists() {
        // Test that command module is accessible
        use super::command;
        let _ = command::execute_status("true");
    }

    #[test]
    fn test_linux_info_module_exists() {
        // Test that linux_info module is accessible
        use super::linux_info;
        let _ = linux_info::detect_distro();
    }
}
