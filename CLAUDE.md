# Factory Game
that's what this is

# Design Goals
a game similar in style to Miner's Haven the Roblox game. 3d with physics and day/night cycle. a concrete square platform which player gets to build on, surrounded by natural environment, an island i suppose. objects are composed out of shapes in a very roblox like fashion, with materials, occasionally transparency, maybe some kind of programmatic texturing. shapes generally not smoothed with neighbours except for what should be soft objects but really there's not gonna be much need for that, but sometimes have like rounded corners on smooth objects.
 shapes are expressed as SDF/CSG trees, baked to meshes with fidget (manifold dual contouring, preserves sharp corners) and rendered through bevy's instanced mesh pipeline. no runtime raymarching. use bevy 0.19.1 and appropriate current crates.
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

Env vars, both inert when unset:
- `FACTORY_SHOT=<secs>` — take a screenshot at that elapsed time, then exit. Lands in `screenshots/`.
- `FACTORY_DAY=<secs>` — length of one day/night cycle, default 240.

So `FACTORY_DAY=24 FACTORY_SHOT=5 cargo run` gives a daylight screenshot without waiting or pressing anything, and `FACTORY_SHOT=17` on the same short day gives a night one.

