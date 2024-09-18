mod flight_loop;
mod loadout;
mod log;
mod plugin;

use xplm::xplane_plugin;

use plugin::PersistentLoadoutPlugin;

/// Re-export the signature of XPLMDebugString as it is needed in the debug macros.
/// By re-exporting we can avoid that users have to import xplm_sys into their plugin.
#[doc(hidden)]
pub use xplm_sys::XPLMDebugString;


xplane_plugin!(PersistentLoadoutPlugin);
