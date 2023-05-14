use std::collections::{HashSet, HashMap};
use owo_colors::OwoColorize;
use rand::prelude::*;
use comfy_table::{Color, Table, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS, Cell, CellAlignment};
use getch_rs::{Getch, Key};

use once_cell::sync::Lazy;

type Position = (usize, usize);
type Board = Vec<Vec<u32>>;

static TILE_COLORS: Lazy<HashMap<u32, (Color, Color)>> = Lazy::new(|| {
    let mut colors: HashMap<u32, (Color, Color)> = HashMap::new();

    colors.insert(2, (Color::Grey, Color::Black));
    colors.insert(4, (Color::Red, Color::Black));
    colors.insert(8, (Color::Green, Color::Black));
    colors.insert(16, (Color::Yellow, Color::Black));
    colors.insert(32, (Color::Blue, Color::Black));
    colors.insert(64, (Color::Magenta, Color::Black));

    colors.insert(128, (Color::Grey, Color::White));
    colors.insert(256, (Color::Red, Color::White));
    colors.insert(512, (Color::Green, Color::White));
    colors.insert(1024, (Color::Yellow, Color::White));
    colors.insert(2048, (Color::Blue, Color::White));
    colors.insert(4096, (Color::Magenta, Color::White));

    return colors;
});

#[derive(Debug)]
pub struct BoardConfig {
    pub width: usize,
    pub height: usize,
    pub count: usize
}
impl Default for BoardConfig {
    fn default() -> Self {
        BoardConfig {
            width: 4,
            height: 4,
            count: 2
        }
    }
}

#[derive(Debug)]
pub enum GameResult { GameOver, Exit, NoMove, NextMove, Reset, UnknownKeyPress }

#[inline]
fn random_tile() -> u32 {
    // 4 Tile (10%), 2 Tile (90%)
    if thread_rng().gen_bool(1.0 / 10.0) == true {
        return 4;
    } else {
        return 2;
    };
}

fn random_board(config: &BoardConfig) -> Result<Board, &'static str> {
    if config.count == 0 { return Err("Empty board!"); }
    if config.count == config.width * config.height { return Err("Full board!"); }
    if config.count > config.width * config.height { return Err("Overflow!"); }

    // get `count` unique positions on the board
    let mut unique_positions: HashSet<Position> = HashSet::<Position>::with_capacity(config.count);
    while unique_positions.len() != config.count {
        let position: Position = (thread_rng().gen_range(0..config.height), thread_rng().gen_range(0..config.width));
        if unique_positions.contains(&position) == false {
            unique_positions.insert(position);
        }
    }

    // allocate board
    let mut board: Board = vec![vec![0; config.width]; config.height];

    // generate board values from positions
    for position in unique_positions {
        board[position.0][position.1] = random_tile();
    }

    return Ok(board);
}

fn move_zeroes_end(array: &mut Vec<u32>) {
    if array.is_empty() { return; }

    let mut j: usize = 0;
    for i in 0..array.len() {
        if array[i] != 0 {
            (array[i], array[j]) = (array[j], array[i]);
            j += 1;
        }
    }
}

fn move_zeroes_start(array: &mut Vec<u32>) {
    if array.is_empty() { return; }

    let mut j: usize = array.len() - 1;
    for i in (0..array.len()).rev() {
        if array[i] != 0 {
            (array[i], array[j]) = (array[j], array[i]);
            j -= 1;
        }
    }
}

fn equal_boards(a: &Board, b: &Board) -> bool {
    if a.len() != b.len() { return false; }

    for row in 0..a.len() {
        if a[row].len() != b[row].len() { return false; }

        for i in 0..a[row].len() {
            if a[row][i] != b[row][i] { return false; }
        }
    }

    return true;
}

enum Keypress { Up, Down, Left, Right, Reset, Quit }
impl TryFrom<Key> for Keypress {
    type Error = &'static str;
    fn try_from(value: Key) -> Result<Self, Self::Error> {

        match value {
            Key::Char('w') | Key::Char('W') | Key::Up => Ok(Keypress::Up),
            Key::Char('s') | Key::Char('S') | Key::Down => Ok(Keypress::Down),
            Key::Char('a') | Key::Char('A') | Key::Left => Ok(Keypress::Left),
            Key::Char('d') | Key::Char('D') | Key::Right => Ok(Keypress::Right),
            Key::Char('r') | Key::Char('R') => Ok(Keypress::Reset),
            Key::Char('q') | Key::Char('Q') | Key::Esc => Ok(Keypress::Quit),
            _ => Err("Invalid Key")
        }

    }
}

#[derive(Debug)]
pub struct Game {
    pub config: BoardConfig,
    board: Vec<Vec<u32>>,
    score: u32
}

impl Game {

    #[inline(always)]
    fn apply_score(&mut self, value: u32) {
        self.score += value;
    }

    pub fn new_game(board_config: Option<BoardConfig>) -> Result<Self, &'static str> {
        let config: BoardConfig = board_config.unwrap_or_default();
        let board: Board = random_board(&config)?;
        return Ok(Self { config, board, score: 0 });
    }

    fn game_over(&self) -> bool {

        // if there are zeroes on the board, its not a game over
        for i in 0..self.config.height {
            for j in 0..self.config.width {
                if self.board[i][j] == 0 { return false; }
            }
        }

        // check if theres a possible move, if there is not then its game over

        // left move
        for row in 0..self.config.height {
            for i in 0..self.config.width {
                if self.board[row][i] == 0 { continue; }

                for j in (i + 1)..self.config.width {
                    if self.board[row][j] == 0 { continue; }
                    if self.board[row][i] == self.board[row][j] { return false; }
                    break;
                }
            }
        }

        // right move
        for row in 0..self.config.height {
            for i in (0..self.config.width).rev() {
                if self.board[row][i] == 0 { continue; }

                for j in (0..i).rev() {
                    if self.board[row][j] == 0 { continue; }
                    if self.board[row][i] == self.board[row][j] { return false; }
                    break;
                }
            }
        }

        // up move
        for column in 0..self.config.width {
            for i in 0..self.config.height {
                if self.board[i][column] == 0 { continue; }

                for j in (i + 1)..self.config.height {
                    if self.board[j][column] == 0 { continue; }
                    if self.board[i][column] == self.board[j][column] { return false; }
                    break;
                }
            }
        }

        // down move
        for column in 0..self.config.width {
            for i in (0..self.config.height).rev() {
                if self.board[i][column] == 0 { continue; }

                for j in (0..i).rev() {
                    if self.board[j][column] == 0 { continue; }
                    if self.board[i][column] == self.board[j][column] { return false; }
                    break;
                }
            }
        }

        return true;
    }

    pub fn play_move(&mut self, getch: &Getch) -> Result<GameResult, Box<dyn std::error::Error>> {

        // game over check
        if self.game_over() == true { return Ok(GameResult::GameOver); }

        // user input
        let input: Key = getch.getch()?;
        let keypress: Keypress = match Keypress::try_from(input) {
            Ok(key) => key,
            Err(_) => return Ok(GameResult::UnknownKeyPress)
        };

        // used to check if the move was `successful`, eliminating reduntant moves
        let board_before_move: Board = self.board.clone();

        match keypress {
            Keypress::Left => self.move_left(),
            Keypress::Right => self.move_right(),
            Keypress::Up => self.move_up(),
            Keypress::Down => self.move_down(),
            Keypress::Quit => return Ok(GameResult::Exit),
            Keypress::Reset => return Ok(GameResult::Reset)
        }

        if equal_boards(&self.board, &board_before_move) == false {
            // move made, add random tile
            self.add_random_tile();
        } else {
            return Ok(GameResult::NoMove);
        }

        return Ok(GameResult::NextMove);
    }

    fn add_random_tile(&mut self) {
        let mut free_tiles: Vec<Position> = vec![];

        for i in 0..self.config.height {
            for j in 0..self.config.width {
                if self.board[i][j] == 0 {
                    free_tiles.push((i, j));
                }
            }
        }

        if free_tiles.len() == 0 { return; } // no free tiles

        // pick & apply random position
        let random_index: usize = thread_rng().gen_range(0..free_tiles.len());
        let random_position: Position = free_tiles[random_index];
        self.board[random_position.0][random_position.1] = random_tile();
    }

    fn move_left(&mut self) {
        // merge from left to right for each row
        for row in 0..self.config.height {
            for i in 0..self.config.width {
                if self.board[row][i] == 0 { continue; } // try next, skip zeroes

                for j in (i + 1)..self.config.width {
                    if self.board[row][j] == 0 { continue; } // try next, skip zeroes

                    // if the first match can be merged, then merge
                    if self.board[row][i] == self.board[row][j] {
                        self.board[row][i] <<= 1;
                        self.apply_score(self.board[row][i]);
                        self.board[row][j] = 0;
                    }

                    break;
                }
            }
        }

        // move all zeros to the end of each row
        self.board.iter_mut().for_each(|row| move_zeroes_end(row));
    }

    fn move_right(&mut self) {
        // merge from right to left for each row
        for row in 0..self.config.height {
            for i in (0..self.config.width).rev() {
                if self.board[row][i] == 0 { continue; }

                for j in (0..i).rev() {
                    if self.board[row][j] == 0 { continue; }

                    if self.board[row][i] == self.board[row][j] {
                        self.board[row][i] <<= 1;
                        self.apply_score(self.board[row][i]);
                        self.board[row][j] = 0;
                    }

                    break;
                }
            }
        }

        // move all zeros to the beggining of each row
        self.board.iter_mut().for_each(|row| move_zeroes_start(row));
    }

    fn move_up(&mut self) {
        // merge from top to bottom for each column
        for column in 0..self.config.width {
            for i in 0..self.config.height {
                if self.board[i][column] == 0 { continue; }

                for j in (i + 1)..self.config.height {
                    if self.board[j][column] == 0 { continue; }

                    if self.board[i][column] == self.board[j][column] {
                        self.board[i][column] <<= 1;
                        self.apply_score(self.board[i][column]);
                        self.board[j][column] = 0;
                    }

                    break;
                }
            }
        }

        // move all zeros to the bottom of each column
        for column in 0..self.config.width {
            let mut j: usize = 0;
            for i in 0..self.config.height {
                if self.board[i][column] != 0 {
                    (self.board[i][column], self.board[j][column]) = (self.board[j][column], self.board[i][column]);
                    j += 1;
                }
            }
        }
    }

    fn move_down(&mut self) {
        // merge from bottom to top of each column
        for column in 0..self.config.width {
            for i in (0..self.config.height).rev() {
                if self.board[i][column] == 0 { continue; }

                for j in (0..i).rev() {
                    if self.board[j][column] == 0 { continue; }

                    if self.board[i][column] == self.board[j][column] {
                        self.board[i][column] <<= 1;
                        self.apply_score(self.board[i][column]);
                        self.board[j][column] = 0;
                    }

                    break;
                }
            }
        }

        // move all zeros to the top of each column
        for column in 0..self.config.width {
            let mut j: usize = self.config.height - 1;
            for i in (0..self.config.height).rev() {
                if self.board[i][column] != 0 {
                    (self.board[i][column], self.board[j][column]) = (self.board[j][column], self.board[i][column]);
                    j -= 1;
                }
            }
        }
    }


    // todo add score / stuff
    pub fn display_game(&self) -> Result<(), Box<dyn std::error::Error>> {

        let mut table: Table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_width(100);

        for i in 0..self.config.height {

            let mut row: Vec<Cell> = vec![];
            for j in 0..self.config.width {
                let tile_value: u32 = self.board[i][j];

                let cell_colors: &(Color, Color) = TILE_COLORS.get(&tile_value).unwrap_or(&(Color::White, Color::Black));

                let mut cell_value: String = String::from("");
                if tile_value != 0 {
                    cell_value.push_str(&tile_value.to_string());
                }


                let cell: Cell = Cell::new(cell_value)
                    .set_alignment(CellAlignment::Center)
                    .fg(cell_colors.0)
                    .bg(cell_colors.1);

                row.push(cell);
            }

            table.add_row(row);
        }

        // print everything
        println!("{}c", 27 as char); // clear (terminal) screen
        println!("{} or {} - Up/Left/Down/Right", "WASD".yellow().bold(), "Arrow Keys".yellow().bold());
        println!("{} - Reset/New Game", "R".cyan().bold());
        println!("{}/{} - Quit", "Q".red().bold(), "Esc".red().bold());
        println!("{}", table);
        println!("{}{}", "Score: ".underline(), self.score.green().bold().underline());

        return Ok(());
    }
}
