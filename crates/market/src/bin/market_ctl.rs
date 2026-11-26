// Drive the game by hand: one argument holding commands separated by ';' (see ctl.rs), e.g.
// market_ctl "click 100 200; wait 500; shot out.png 0 0 960 540"
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    script: Vec<String>,
}

fn main() {
    let script = Cli::parse().script.join(" ");
    let mut g = hf_market::game::Game::find();
    print!("{}", hf_market::ctl::run(&mut g, &script));
    println!("ok, game client at ({}, {}) size {}x{}", g.x, g.y, g.w, g.h);
}
