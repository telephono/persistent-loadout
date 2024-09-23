/// Writes a message to the developer console and Log.txt file, with a newline
#[macro_export]
macro_rules! debugln {
    ($($arg:tt)*) => ({
        let message: String = std::fmt::format(std::format_args!($($arg)*));
        let formatted_string = format!("{}: {message}\n", $crate::plugin::NAME);

        match std::ffi::CString::new(formatted_string) {
            Ok(c_str) => unsafe { $crate::XPLMDebugString(c_str.as_ptr()) },
            Err(_) => unsafe { $crate::XPLMDebugString("[xplm] Invalid debug message\n\0".as_ptr() as *const _) }
        }
    });
}
