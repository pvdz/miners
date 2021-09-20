use std::env;

pub struct Options {
    pub seed: u64,
    pub mutation_rate_genes: f32,
    pub mutation_rate_slots: f32,
    pub speed: u64,
    pub visual: bool,
}

pub fn parse_cli_args() -> Options {
    // Defaults:
    let mut options = Options {
        seed: 0,
        speed: 10,
        mutation_rate_genes: 5.0,
        mutation_rate_slots: 5.0,
        visual: true,
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
