use std::env;

pub struct Options {
    pub batch_size: u8,
    pub mutation_rate_genes: f32,
    pub mutation_rate_slots: f32,
    pub mutate_from_best: bool, // Mutate a new batch from the overall best or the last winner?
    pub reset_rate: u32,        // Reset every this many generated miners
    pub reset_after_noop: bool, // Only reset after that many miners did not yield a new best?
    pub seed: u64,
    pub speed: u64,
    pub visual: bool,

    // Debugging
    pub paint_ten_lines: bool, // Draw grids at every 10th line/col
    pub paint_zero_zero: bool, // Draw hash for the 0,0 coord
    pub paint_miner_ids: bool, // Draw biome index for other biome miners rather than emoji
    pub paint_empty_world: bool, // Always draw empty tiles instead of the world
    pub hide_world_oob: bool, // Do not draw the world that doesn't explicitly exist in memory
    pub hide_world_ib: bool, // Do not draw the world that explicitly exists in memory (only oob)
    pub paint_visited: bool, // Paint the number of times the miner visited a tile, in the world view?
    pub paint_visited_bool: bool, // If the miner visited a tile, paint that tile so you can see? Not a count, just a yes/no.
}

pub fn parse_cli_args() -> Options {
    // Defaults:
    let mut options = Options {
        batch_size: 10,
        mutation_rate_genes: 5.0,
        mutation_rate_slots: 5.0,
        mutate_from_best: false,
        seed: 210114, // 0 is random
        speed: 1,
        reset_rate: 500,
        reset_after_noop: true,
        visual: true,

        // Debug
        paint_ten_lines: false,
        paint_zero_zero: false,
        paint_empty_world: false,
        paint_miner_ids: false,
        hide_world_oob: false,
        hide_world_ib: false,
        paint_visited: true,
        paint_visited_bool: false,
    };

    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let mut index = 1; // The first one is the binary path so let's skip that :)
    while index < args.len() {
        match args[index].as_str() {
            "--seed" => {
                index = index + 1;
                options.seed = args[index].trim().parse::<u64>().unwrap_or(0);
                if options.seed == 0 {
                    panic!("Seed must be a non-zero positive integer");
                }
            }
            "--visual" => {
                options.visual = true;
            }
            "--no-visual" => {
                options.visual = false;
            }
            _ => {
                println!("Unknown parameter: {}", args[index]);
                panic!();
            }
        }

        index = index + 1;
    }

    options
}
