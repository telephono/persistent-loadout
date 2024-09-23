#[macro_use]
extern crate xplm;

mod flight_loop;
mod loadout;
mod plugin;

use plugin::PersistentLoadoutPlugin;
xplane_plugin!(PersistentLoadoutPlugin);
