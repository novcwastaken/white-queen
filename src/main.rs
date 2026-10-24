use colored::Colorize;
use std::env;

fn main() {
    const STARTING_BET_HINT: &str = "Your starting bet must be an integer above 0! Pass it as an argument to the white-queen executable.";

    let args: Vec<String> = env::args().collect();

    // Validate the starting bet argument
    if args.len() < 2 {
        println!("{}", STARTING_BET_HINT.red());
        return;
    }

    let starting_bet: u32;
    match args[1].trim().parse::<u32>() {
        Err(_) => {
            println!("{}", STARTING_BET_HINT.red());
            return;
        }
        Ok(num) => {
            if num == 0 {
                println!("{}", STARTING_BET_HINT.red());
                return;
            } else {
                starting_bet = num;
            }
        }
    }

    // Welcome
    println!(
        "{}{}{}",
        "Welcome to ".bright_green(),
        "WHITE QUEEN".magenta().bold(),
        "!".green()
    );
    println!(
        "{}{}{}",
        "Your starting bet is ".bright_green(),
        format!("{} Doubloons", starting_bet).yellow().bold(),
        ".".bright_green()
    );
}
