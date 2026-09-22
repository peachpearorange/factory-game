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

# Designing Machines
Models are written in code. There are no mesh assets. Build out of parts by default.

Parts are not just boxes. Rods and balls, wedges for roofs, ramps and prows, free rotation on every axis, `span` between two points, and `ringed`/`repeated` copies will give you barrels, domes, cones, arches, spirals, fans of blades, tapers stacked out of shrinking slabs — curves and organic masses included, in the faceted Roblox way that suits this game. Reach for an SDF only when a shape genuinely needs smooth blending between volumes, and never fight the SDF to get a shape parts would have given you.

Materials all live in `material.rs`. A `Finish` is a surface (Painted, Plastic, Rock, Wood, Planked, Metal, Glass, Cloth) plus a colour and a glow, and the named consts — `TIMBER`, `STONE`, `GOLD`, `BRASS`, `SOOT`, `GLASS` and the rest — are the palette. Add to that palette rather than declaring a loose `LinearRgba` next to a builder. Derive variants with `.tinted()`, `.shaded()`, `.lit()`, `.veiled()`. Alias the ones a machine uses at the top of its builder:

```rust
let (stone, gold, timber) = (material::STONE, material::GOLD, material::TIMBER);
```

The finish is the constructor, so a part can never lose its material:

```rust
stone.slab(Vec3::new(1.70, 0.70, 2.90)).on(Vec3::new(-1.15, MINE_CREST, 0.0)).solid()
timber.beam(MINE_HEAD - MINE_DECK, 0.14).span(foot, head)
material::STEEL.rod(0.48, 0.12).at(hoist).rolled(FRAC_PI_2)
```

Forms are `slab`, `cube`, `beam`, `rod`, `ball`, `wedge`, plus `faces` and `shell` when a shape wants coordinates of its own:

```rust
stone.faces([
  vec![low, east, north],
  vec![low, peak, east],
  vec![east, peak, north],
  vec![north, peak, low]
])
```

`faces` takes polygons — any number of corners each, fan-triangulated and flat-shaded — and `shell` takes a flat run of points read three at a time as triangles. Wind each face counter-clockwise seen from outside; the normal comes from the winding, so a reversed face turns invisible under back-face culling. The part's size is the bounding box of the points, so `on`, `under` and `span` keep working, and resizing scales the points. Its collider is the convex hull, so author a concave shape as several `.solid()` parts.

Sizes are full extents, never halves. Place with `at` (centre), `on` (bottom rests here), `under` (top hangs here) or `span(a, b)` (stretches between two points, working out length and rotation itself — use it for braces, ladders, cables, anything diagonal). Then `.solid()` for a collider, `.tilted/.rolled/.turned/.spun` to rotate.

Where a part genuinely repeats, say so once. `group([...])` makes an Assembly that folds its transform into its parts:

```rust
group([timber.beam(CHUTE_FLOOR, 0.14).on(Vec3::new(0.46, 0.0, 0.50))])
  .mirrored(Axis::X)
  .mirrored(Axis::Z)
  .at(Vec3::X * DROPPER_BACK)
```

`mirrored(axis)` is a true reflection, so a tilted roof mirrors its tilt. `repeated(n, step)` gives runs of sleepers, rivets, palings. `ringed(n, radius)` gives bolt circles, spokes, staves. A sub-assembly — a cart, a winch, a chimney — should be its own `fn -> Assembly` that the machine places and rotates as a unit.

A group's list holds anything part-shaped — a `Part`, an `Option` of one, another Assembly — so a machine is assembled by naming its pieces and listing them, never by chaining iterators:

```rust
group([basin, ring, bolts, pump, steps, cover, towel, panel, duck])
```

`.with(more)` adds one such piece to an Assembly, which is how an optional or one-off piece joins a group. `radial(angle, radius)` places a part or an assembly out along a ray and turns it to face that way, so anything authored along +X — a stave, a jet, a control panel — is written once at its own radius. `ring(n, |spoke| ...)` is `ringed` for spokes that differ: the closure returns the Assembly for that spoke, so an exception (a weir where a stave would be, one bracket left off) is a plain `if` inside it:

```rust
part::ring(TUB_STAVES, |spoke| {
  let weir = spoke == TUB_WEIR_STAVE;
  group([cedar.slab(size).on(Vec3::X * TUB_RADIUS).solid()])
    .with(weir.then(|| brass.slab(cap).on(Vec3::new(TUB_RADIUS, TUB_LEDGE, 0.0))))
    .with((!weir).then(|| group([coping, hoop(TUB_HOOP_LOW), bench, jet])))
})
```

This is for things that are actually identical — the four legs of a coop, a run of sleepers. It is not an instruction to make everything symmetric. Boulders of differing girth, a lamp hung off one side, a door that is not centred, one shutter askew: hand-place those, and let them be uneven on purpose. A machine that is perfectly mirrored on every axis looks machine-made in the wrong way. Reach for the combinators to avoid copy-paste, not to flatten character out of a model.

Name every dimension as a machine-prefixed `SCREAMING_SNAKE` const at file level, and name builders and closures after the physical object (`gold_mine`, `trestle`, `jamb`, `mast`, `yoke`).

The machine itself is one `#[assoc(...)]` block on a `MachineKind` variant, appended to `MachineKind::ALL`. `parts` for a built model, `shape` + `paint` for an SDF one, `finish` for its surface. Local origin is at ground and at the footprint centre; `CELL` is 2.0 and `footprint` counts cells. Belts run along +X: set `carries_belt` and include `belt_deck(belt_half(cells))`, and `place()` adds the belt, slats and upgrader sensor. Non-mesh trimmings — lights, particles, glass, SDF colliders — go in the per-kind match in `place()`.

Do not be lazy. A machine that is a box with a pipe on it is a failure. Put the effort in: legs and footplates, bracing and gussets, rivets and bolt circles, hinges, ladders, railings, handwheels, gauges and dials, vents, chains, warning stripes, a maker's plate, pipework that actually goes somewhere, wear and soot where wear and soot belong. Use the whole palette in one machine — timber against iron against brass against glass — because per-part materials are what make it read as a built object. Go extravagant and characterful over clean and minimal every time. The parts API is cheap; there is no excuse for a plain model.

Then look at it. `tools/showcase <name>` writes twelve shots, read them, and adjust the consts. A model that reads fine in code is routinely wrong in silhouette, so never call one finished without seeing it.

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

