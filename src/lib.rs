// Copyright (c) 2024 telephono
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

#[macro_use]
extern crate xplm;

mod flight_loop;
mod loadout;
mod plugin;

xplane_plugin!(plugin::PersistentLoadoutPlugin);
