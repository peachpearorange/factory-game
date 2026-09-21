# Factory Game
that's what this is

# Design Goals
a game similar in style to Miner's Haven the Roblox game. 3d with physics and day/night cycle. a concrete square platform which player gets to build on, surrounded by natural environment, an island i suppose. objects are composed out of shapes in a very roblox like fashion, with materials, occasionally transparency, maybe some kind of programmatic texturing. shapes generally not smoothed with neighbours except for what should be soft objects but really there's not gonna be much need for that, but sometimes have like rounded corners on smooth objects.
  use bevy 0.19.1 and appropriate current crates.
player unlocks and buys machines, they can be quite wacky but generally have a conveyor belt through if they're upgraders, and perform a change to the object put in. can be far broader than in Miner's Haven as "ores" may be more diverse things such as dragon eggs which go in a hatching machine and other upgraders, to ultimately produce baby dragons with different properties.
so different properties there like putting things on fire or making things wet or radioactive and whatnot. some machines respond to these.
ores within certain distance of camera have a popup above them with value and colored symbolic icons(svg perhaps) indicating properties that they have. when an ore sells in a furnace, a similar number popup shows up indicating what it sold for.
 player has an actual player model with a simple collider in a robloxy way and walks around. there may be special challenges to produce objects with specific properties and player should get to mouse over a created object and see some overview popup of what it is and which effects it has and what it's worth.
a popup can be opened for the store where player can buy machines, but most machines are unlocked through completing some sort of challenge
machines placed on a grid. standard conveyor belt is 2x2 in the grid. hitboxes of machines rendered as blue transparent boxes with lines at edges, turns red if player is placing a machine that collides with an existing machine.

# Code
avoid comments. just have code self-explanatory by names and structure. concise. use let chains. if there's repetition, make helpers.

# Notes
`world::GROUND` is y = 0, the exact top of the concrete platform. Author every shape and spawn position relative to it so meshes rest on the ground.

One env var, `FACTORY`, holds a JSON5 `opts::Opts` — braces, unquoted keys, missing fields take their defaults, an unknown field panics at startup with the field list. Unset means every default, so the game runs as usual.

- `day: <secs>` — length of one day/night cycle, default 240.
- `time: <secs>` — how far into the cycle the game starts. 0 is sunrise, a quarter of `day` is noon, three quarters is midnight.
- `trade: <secs>` — how long the trade boat stays away between visits, default 170. Set it low to watch it sail in.
- `money: <amount>` — what the purse starts with, default 600. Set it high to test machines without earning them.
- `shot: <secs>` — take a screenshot at that elapsed time, then exit. Lands in `screenshots/`.
- `showcase: '<machine name fragment>'` — see below.

So `FACTORY='{day: 24, time: 6, shot: 2}' cargo run` gives a noon screenshot straight away, and `time: 17` gives a night one without waiting through the day. Add a field to `Opts` for each new knob rather than reaching for a second var.

`tools/showcase <machine name fragment>` is the way to look at a machine. It places it at the origin, flanked by conveyors if it carries a belt, hides the HUD, freezes the player camera, and orbits 35/125/215/305 degrees at dawn, noon and night, writing twelve shots to `screenshots/showcase/<machine>-<moment>-<angle>deg.png` before exiting. `FACTORY="{showcase: 'orewash'}"` alone does the same without the script. The fragment matches the spec name ignoring case and punctuation, so `orewash` finds The Orewash.

