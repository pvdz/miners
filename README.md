# Miners

This is my first project in Rust, something to get started with. It's an idle game that is a [Langton Ant](https://en.wikipedia.org/wiki/Langton%27s_ant) with [Genetic Algorithm](https://en.wikipedia.org/wiki/Genetic_algorithm) leading to [emergent gameplay](https://en.wikipedia.org/wiki/Emergent_gameplay).

A Langton Ant is a "cellular automaton", basically an entity moving along a cell based grid where the movement is entirely dictated by the cell it's on, or its neighbors.

A Genetic Algorithm attempts to optimize a problem by starting at a random (or not so random) point in the search space and incrementally improving the score ("cost function"). Very similar to how genetics work in the real world, "survival of the fittest" and all that.

Combined there's an emergent gameplay where you search for the miner that collects the most points. Just don't expect much from it :)

## Inspiration

I drew my inspiration from [boxcar2d](https://rednuht.org/genetic_cars_2/), an enticing time sink where a randomly generated vehicle traverses a randomly generated world. At the end of a run the best car is picked, copied a few times with minor modifications, and the next run starts.

The "best car" is the "cost function" which in boxcar simply comes down to the distance traveled. The genetics are the phsyical appearance of the car; wheel size, weight, form factor.

What I really like about boxcar is the emergent gameplay that develops by combining two unrelated things; a physics engine and randomly combining some moving parts. The physics weren't created to be a game. They're just physics.

The "game" aspect comes from waiting to see whether any of the new set of cars that was generated from the previous best, is actually better than its parent. This is what I kinda wanted to do with Miners as well.

You would be able to turn the knobs to tweak what got improved but you'd still have to run the result to see whether it was in fact better.

## The game

Miners was set out to be about a miner in a world of rocks. Every time the miner hits a rock it chips it down a bit, until it crumbles. When that happens a diamond may appear. Hey, collectables. Crafting!

The miner can have items. Some items have an auto-charging mechanism, others are passives that activate with certain actions. There's one particularly special item which launches drones. Drones take a chunk of your energy and then set out in the world to do what you do, but less efficient.

I wanted to create a crafting mechanism as well. So when you collect enough materials the miner starts creating special drones.

There's a total of four of these special drones to build. And a castle. But I'll leave you to figure that out for yourself.

When running multiple instances for the GA it will show you the position of the other miners, which will be in the same world as you are. (You can configure this visualization; it's not as inspiring as I hoped it to be, like how boxcar can do it).

## Features

- Langton Ant as a game
- Genetic Algorithm
- Automatic Crafting System
- Wells of water
- Sand castles
- Aid drones
- Points
- Gamification!
- Emergent gameplay
- Never ending sink hole
- Kids that call your carefully crafted miners and their drones "mice" (:oof:)

But seriously

- Rust based "game"
- Runs in the terminal
- Runs in the browser (same code)
- Minimal dependencies
  - `serde` for JSON
  - Whatever is required to compile to WASM
  - `rand` (for PRNG)
  - Everything else is hand crafted, for better or worse.
- Can serialize and deserialize the state
- Theoretically infinite world
- Camera that can show arbitrary portion of the game world, can be controlled, can auto-follow
- Fixed procedurally generated world (only uses seeded rng, tiles are generated by algorithm)
- Rudimentary cli controls that work the same in the browser (shares same logic)
- Bunch of crappy code from a Rust noob learning the ropes

## Running it

Run it in CLI:

```
cargo run
```

Or my standard line is something like
```
RUST_BACKTRACE=1 cargo run -- --seed 20220314 --batch-size 10 --no-visual
RUST_BACKTRACE=1 cargo run -- --seed 210143 --batch-size 10 --miner '[210143,43.0,129.0,0.0,8.0,0.0,"..DDDDDDDDd.h.dd.EEE.JJJEP.EdPhh"]' --visual --batch-size 10
```

See below for option details.

To generate the wasm binary:

```
wasm-pack build --target web
```

Then open `./html/index.html` in a local browser to run it (you can run a local server to serve it, or whatever).

## Options and config

The CLI (and json in web) have the following options (see `options.rs`):

- `--seed <number>`: Initializes the starting seed for this world. By default it will generate a pseudo-random world seed
- `--visual`: Set `options.visual = true`, which will enable visual mode. Runs slower but nicer to look at.
- `--no-visual`: Set `options.visual = false`, which will disable visual mode. Runs faster but a little boring.
- `--batch-size <number>`: Set the number of miners should be generated per batch
- `--miner <string>`: The miner code for the first ("root") miner instance. The string is a specific json, with seed, initial helix values, and starting items. Example value: `[210143,43.0,129.0,0.0,8.0,0.0,"..DDDDDDDDd.h.dd.EEE.JJJEP.EdPhh"]`

The `options.rs` file contains many more options and there are more in `app_state.rs`. Sorry for the mess there, the app state was a last addition that wasn't properly fleshed out.

### options

- `batch_size`: number of miners to generate per batch
- `initial_miner_code`: root miner code at start of app
- `mutation_rate_genes`: offspring mutation rate of genetics
- `mutation_rate_slots`: offspring mutation rate of items
- `mutate_from_best`: mutate a new batch from the overall best or the last winner?
- `reset_rate`: reset the "best miner" every this many generated miners, basically puts the search in a random new spot, hoping it leads to better answers when the current search gets stuck in a local plateau
- `reset_after_noop`: only reset after `reset_rate` miners did not yield a new best? Rather than absolute count
- `return_to_move`: while `true`, you need to press return to step forward. Useful for debugging
- `seed`: initial world seed
- `speed`: tick/frame delay, cli only, this is the value passed on to `thread.sleep()`
- `cost_increase_rate`: the rate after which the overall difficulty cost goes up
- `cost_increase_interval`: the last time the overall difficulty cost went up
- `frame_skip`: only print and read input every this many ticks
- `frames_now`: current progress of the frame skip
- `visual`: print world?
- `sandrone_pickup_count`: sandrone will pick up miner after putting down this many push tiles
- `sandcastle_area_limit`: sandrone will permanently stop building the wall after the castle area is at least this big
- `html_mode`: print the world in html rather than terminal ansi?
- `show_biomes`: when printing the world should it print all miners in the current batch? Confusing but fun!
- `visible_index`: when visual=true, which biome are we painting?
// Debugging stuff
- `paint_ten_lines`: print a special icon every ten tiles of the world from the origin?
- `paint_zero_zero`: print a special icon at the origin (0,0) ? 
- `paint_miner_ids`: print miner world index ids for each miner printed?
- `paint_empty_world`: always draw empty tiles instead of the world
- `hide_world_oob`: do not draw the world that doesn't explicitly exist in memory
- `hide_world_ib`: do not draw the world that explicitly exists in memory (only paint oob tiles which are procedurally generated) 
- `paint_visited`: paint the number of times the miner visited a tile, in the world view? 
- `paint_visited_bool`: if the miner visited a tile, paint that tile differently so you can see? Not a count, just a yes/no.
- `paint_colors`: film noir?
- `paint_bg_colors`: disable background colors while keeping foreground colors
- `paint_fg_colors`: disable foreground colors while keeping background colors

### app state

- `startup`: just a flag to indicate that this is the first loop
- `best_miner`: (Helix, u64, usize, usize, Inventory), helix, seed, generated grid width height, inventory
- `trail_lens`: something I was working with but dropped
- `instance_rng_seeded`: seeded with input seed
- `instance_rng_unseeded`: seeded from random input, so different for each app start
- `best_min_x`: world size of best miner
- `best_min_y`: world size of best miner
- `best_max_x`: world size of best miner
- `best_max_y`: world size of best miner
- `viewport_offset_y`: when printing the world, this is the top-left corner
- `viewport_offset_x`: when printing the world, this is the top-left corner
- `viewport_size_w`: width of the viewport
- `viewport_size_h`: height of the viewport
- `center_on_miner_next`: when set, a one time action, done this way cause otherwise we need to juggle the miner position everywhere
- `auto_follow_miner`: always make sure the miner is in viewport?
- `auto_follow_buffer_min`: once the miner moves closer than this many tiles to the border
- `auto_follow_buffer_max`: change the viewport to make it this many tiles instead
- `stdin_channel`: in cli mode this is the other worker
- `delay`: delay is `thread::sleep` driven which won't work in main-web-thread so it's cli only
- `cost_increase_value`: see `options.cost_increase_rate` and `options.cost_increase_interval`
- `total_miner_count`: number of miners that have been generated this app instance
- `current_miner_count`: number of miners that have been generated since last reset
- `miner_count_since_last_best`: number of miners that have been generated since best was updated last
- `start_time`: ms epoch start of this app instance
- `pause_after_ticks`: pause the first world after this many ticks. Tool for debugging
- `stats_last_second`: track last whole second that stats were updated
- `stats_last_biome_ticks`: ticks at last second
- `stats_last_ticks_sec`: ticks passed since previous second
- `stats_total_batches`: total runs done
- `stats_total_batch_loops`: total loops done
- `stats_total_biome_ticks`: total ticks done
- `batch_ticks`: ticks in current batch
- `last_match_loops`: ticks when last printed world
- `has_energy`: is there at least one biome left with a miner that has energy? Current batch ends when this remains false.
- `non_visual_print`: ehhh. timestamp at last frame that was printed? :)
- `reset`: at user request this is true and the next loop will reset the world seed and miners
- `load_best_as_miner_zero`: at user request this is true and the next loop will reset the best miner to the (app) intiial best miner

## Web

After you compile the binary into WASM you can load it into the web page.

The web page has a small interface hooking up the necessary bits and pieces. It allows you to start/pause/stop, set the config, use all the controls you can use in the CLI, and run at pretty much the same non-visual speed. Visual speed is a little slower.

I initially developed it for the CLI and only made it run in the web long after it was up and running properly. As such the game was shoehorned into a web app so it's not using the regular best practices, whatever those might be. In particular, the game awaits a promise to resolve before continuing with the next frame(s). No ideal but works fine for this POC.

Where the CLI generates a string to print the world viewport and the controls, the web does exactly the same. It takes this generated string and dumps into the `.innerHTML` of an html element verbatim.

## Tests

Yes. You should always write tests.

## Future

Eh. This project has none.

There's a few problems but the biggest problem of them all is that I'm lacking inspiration to build this out to a sensible game. Beyond that there's some practical drawbacks;

- The code was written for terminal printing but that severely limits the graphic capabilities
- The limits around a game around a Langton Ant are proving to be too much to make it fun for me to continue working on it
- The code can do with a cleanup now that I know more about Rust
- The cleanup will not be clean as I don't know Rust well enough, so what's the point
- I'd rather work on a new toy while improving my Rust
- The game was not written for web, the way I shoehorned it into the web is far from ideal
- No clear separation of Options vs State (kinda because State was a last minute thing where Options was around from the start)
- Crappy global management in Rust, no object model whatsoever, etc.
- Lots of inefficient string handling, never meant to be fast, just fun
- No inspiration. Right mindset, wrong setting. Not sure how to buid it out. So it's holding me back from working on it.

Okay this was fun bai
