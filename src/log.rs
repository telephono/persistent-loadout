/// Writes a message to the developer console and Log.txt file, with a newline
#[macro_export]
macro_rules! debugln {
    ($($arg:tt)*) => ({
        let mut formatted_string: String = std::fmt::format(std::format_args!($($arg)*));
        formatted_string.push_str("\n");
        let formatted_string = format!("{}: {}", $crate::plugin::NAME, formatted_string);

        match std::ffi::CString::new(formatted_string) {
            Ok(c_str) => unsafe { $crate::XPLMDebugString(c_str.as_ptr()) },
            Err(_) => unsafe { $crate::XPLMDebugString("[xplm] Invalid debug message\n\0".as_ptr() as *const _) }
        }
    });
}