# Persistent Loadout Plugin for Shenshee's B720

This plugin saves the Auto Brake, Auto Thrust, HF antenna and VOR/INS/FMC options on leaving
the aircraft or changing the aircraft's livery. The fuel weight is also saved.

The settings are saved on a per-livery basis in X-Plane's `Output/B720` folder.
The folder structure is as follows:

```
Output/B720/<Model>/<Livery>/persistent-loadout.json
```

These settings are restored when you load into the aircraft or after changing it's livery.

## Installation

To install, download the latest [release](https://github.com/telephono/persistent-loadout/releases), extract and
copy the `persistent-loadout` folder to the aircraft's `plugins` directory.

## Restore Default Options

If you want to reset the options for a particular livery, just remove the corresponding folder
from inside the `Output/B720/<Model>/` folder.
