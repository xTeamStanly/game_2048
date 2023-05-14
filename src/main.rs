use std::error::Error;
use getch_rs::Getch;
use owo_colors::OwoColorize;
mod game;
use game::{Game, BoardConfig, GameResult};

// parse arguments into board configuration
// if anything `bad` happens just use default configuration
fn parse_args(args: &Vec<String>) -> Option<BoardConfig> {
    if args.len() != 3 {
        println!("Not enought arguments. Using default configuration.");
        return None;
    }

    let numbers: Vec<usize> = args.iter().filter_map(|s| s.parse().ok()).collect();
    if numbers.len() != 3 {
        println!("Invalid arguments. Using default configuration.");
        return None;
    }

    return Some(BoardConfig { width: numbers[0], height: numbers[1], count: numbers[2] });
}

fn print_usage() {
    println!("{}: rust_2048 [CONFIG] [FLAGS]", "Usage".green());
    println!();
    println!("{} - {} {} {}", "Config".green(), "NUMBER".bright_red(), "NUMBER".bright_yellow(), "NUMBER".bright_magenta());
    println!(" - consists of three numbers");
    println!(" - {} - Width of the grid", "Grid width".bright_red());
    println!(" - {} - Height of the grid", "Grid height".bright_yellow());
    println!(" - {} - Number of filled in tiles", "Filled count".bright_magenta());
    println!(" - {}: {}", "default value".underline(), "4 4 2".bold());
    println!();
    println!("Flags:");
    println!(" {}, {} - Displays the help message", "-h".bright_blue(), "--help".bright_blue());
    println!();
}

fn main() -> Result<(), Box<dyn Error>> {
    let getch: Getch = Getch::new();
    let config: BoardConfig;

    let should_print_usage: bool = std::env::args().map(|x| x.trim().to_lowercase()).any(|x| x == "--help" || x == "-h");
    if should_print_usage {
        print_usage();
        return Ok(());
    }

    let args: Vec<String> = std::env::args().skip(1).take(3).collect();
    if args.len() == 0 {
        config = BoardConfig::default();
    } else {
        config = parse_args(&args).unwrap_or_default();
    }

    let mut game: Game = Game::new_game(Some(config))?;
    game.display_game()?;

    loop {
        let game_result: GameResult = game.play_move(&getch)?;
        match game_result {
            GameResult::Exit => { break; },

            GameResult::Reset => {
                game = Game::new_game(Some(game.config))?;
                game.display_game()?;
            },

            GameResult::GameOver => {
                game.display_game()?;
                println!("{}", "--- Game Over ---".red());
                break;
            },

            GameResult::NextMove => {
                game.display_game()?;
                println!("{}", "--- Nice Move ---".green());
            },

            GameResult::UnknownKeyPress => {
                game.display_game()?;
                println!("{}", "--- Invalid key ---".red());
            },
            GameResult::NoMove => {
                game.display_game()?;
                println!("{}", "--- Unnecessary move ---".red());
            }
        }
    }

    return Ok(());
}
