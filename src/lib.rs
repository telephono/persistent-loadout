mod flight_loop;
mod loadout;
mod plugin;

use xplm::xplane_plugin;

use plugin::PersistentLoadoutPlugin;

xplane_plugin!(PersistentLoadoutPlugin);
