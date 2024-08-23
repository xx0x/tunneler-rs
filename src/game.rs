// UGLY CODE, JUST A PLAYGROUND FOR LEARNING RUST

use crossterm::event::{Event, KeyCode};
use crossterm::style::SetForegroundColor;
use crossterm::{
    cursor, event, execute,
    style::{Color, PrintStyledContent, Stylize},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io::{stdout, BufWriter, Stdout, Write};
use std::time::{Duration, Instant};

type Screen = BufWriter<Stdout>;

const PLAYER_SPRITE_UP: [&str; 5] = ["D Y D", "DGYGD", "DGYGD", "DGGGD", "D   D"];
const PLAYER_SPRITE_DOWN: [&str; 5] = ["D   D", "DGGGD", "DGYGD", "DGYGD", "D Y D"];
const PLAYER_SPRITE_LEFT: [&str; 5] = ["DDDDD", " GGG ", "YYYG ", " GGG ", "DDDDD"];
const PLAYER_SPRITE_RIGHT: [&str; 5] = ["DDDDD", " GGG ", " GYYY", " GGG ", "DDDDD"];

const FIRE_SPRITE_V: [&str; 5] = ["     ", "  R  ", "  R  ", "  R  ", "     "];
const FIRE_SPRITE_H: [&str; 5] = ["     ", "     ", " RRR ", "     ", "     "];

const GAME_WIDTH: i32 = 1000;
const GAME_HEIGHT: i32 = 1000;

const SPRITE_X_OFFSET: i32 = -2;
const SPRITE_Y_OFFSET: i32 = -2;

const BORDER_SIZE: i32 = 1;

fn draw_sprite(screen: &mut Screen, sprite: &[&str], base_color: Color, x: i32, y: i32) {
    let (width, height) = terminal::size().unwrap();
    for (dy, row) in sprite.iter().enumerate() {
        for (dx, ch) in row.chars().into_iter().enumerate() {
            let new_x = x + dx as i32 + SPRITE_X_OFFSET;
            let new_y = y + dy as i32 + SPRITE_Y_OFFSET;

            if new_x >= BORDER_SIZE && new_x < width as i32 - BORDER_SIZE && new_y >= BORDER_SIZE && new_y < height as i32 - BORDER_SIZE{
                let styled_char = match (ch, base_color) {
                    ('G', Color::Green) => " ".on_green(),
                    ('G', Color::Blue) => " ".on_blue(),
                    ('G', _) => continue,
                    ('D', Color::Green) => " ".on_dark_green(),
                    ('D', Color::Blue) => " ".on_dark_blue(),
                    ('D', _) => continue,
                    ('R', _) => " ".on_red(),
                    ('Y', _) => " ".on_yellow(),
                    _ => continue, // Any other character is treated as empty space
                };
                screen
                    .execute(cursor::MoveTo(new_x as u16, new_y as u16))
                    .unwrap();
                screen.execute(PrintStyledContent(styled_char)).unwrap();
            }
        }
    }
}

#[derive(Clone)]
enum Rotation {
    Up,
    Down,
    Left,
    Right,
}

struct Position {
    x: i32,
    y: i32,
    rotation: Rotation,
}

impl Position {
    fn move_by(&mut self, x: i32, y: i32) {
        if self.x + x >= 0 && self.x + x < GAME_WIDTH {
            self.x += x;
            if x > 0 {
                self.rotation = Rotation::Right;
            } else if x < 0 {
                self.rotation = Rotation::Left;
            }
        }
        if self.y + y >= 0 && self.y + y < GAME_HEIGHT {
            self.y += y;
            if y > 0 {
                self.rotation = Rotation::Down;
            } else if y < 0 {
                self.rotation = Rotation::Up;
            }
        }
    }

    fn move_to(&mut self, x: i32, y: i32) {
        if x >= 0 && x < GAME_WIDTH {
            self.x = x;
        }
        if y >= 0 && y < GAME_HEIGHT {
            self.y = y;
        }
    }
}

struct Fire {
    position: Position,
}

impl Fire {
    fn draw(&self, screen: &mut Screen) {
        match self.position.rotation {
            Rotation::Up => {
                draw_sprite(screen, &FIRE_SPRITE_V, Color::Red, self.position.x, self.position.y)
            }
            Rotation::Down => {
                draw_sprite(screen, &FIRE_SPRITE_V, Color::Red, self.position.x, self.position.y)
            }
            Rotation::Left => {
                draw_sprite(screen, &FIRE_SPRITE_H, Color::Red, self.position.x, self.position.y)
            }
            Rotation::Right => {
                draw_sprite(screen, &FIRE_SPRITE_H, Color::Red, self.position.x, self.position.y)
            }
        }
    }
    fn update(&mut self) {
        // Move the fire
        match self.position.rotation {
            Rotation::Up => self.position.move_by(0, -1),
            Rotation::Down => self.position.move_by(0, 1),
            Rotation::Left => self.position.move_by(-1, 0),
            Rotation::Right => self.position.move_by(1, 0),
        }
    }
}

struct Player {
    position: Position,
    color: Color,
    score: u32,
}

impl Player {
    fn new(color: Color) -> Player {
        Player {
            position: Position {
                x: 0,
                y: 0,
                rotation: Rotation::Up,
            },
            color,
            score: 420,
        }
    }

    fn draw(&self, screen: &mut Screen) {
        let sprite = match self.position.rotation {
            Rotation::Up => PLAYER_SPRITE_UP,
            Rotation::Down => PLAYER_SPRITE_DOWN,
            Rotation::Right => PLAYER_SPRITE_RIGHT,
            Rotation::Left => PLAYER_SPRITE_LEFT,
        };
        draw_sprite(screen, &sprite, self.color, self.position.x, self.position.y);
    }

    fn move_by(&mut self, x: i32, y: i32) {
        self.position.move_by(x, y);
    }

    fn move_to(&mut self, x: i32, y: i32) {
        self.position.move_to(x, y);
    }
}

pub struct Game {
    players: Vec<Player>,
    quit: bool,
    start_time: Instant,
    fires: Vec<Fire>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            players: vec![Player::new(Color::Blue), Player::new(Color::Green)],
            quit: false,
            start_time: Instant::now(),
            fires: Vec::new(),
        }
    }

    fn move_player_by(&mut self, player: usize, x: i32, y: i32) {
        self.players[player].move_by(x, y);
    }

    fn move_player_to(&mut self, player: usize, x: i32, y: i32) {
        self.players[player].move_to(x, y);
    }

    fn make_fire(&mut self, player: usize) {
        let pos = &self.players[player].position;
        let fire = Fire {
            position: Position {
                x: pos.x,
                y: pos.y,
                rotation: pos.rotation.clone(),
            },
        };
        self.fires.push(fire);
    }

    fn draw_frame(&self, screen: &mut Screen) {
        let border = " ".on_dark_grey();
        let (width, height) = terminal::size().unwrap();

        // Draw top and bottom borders
        for x in 0..width {
            screen.execute(cursor::MoveTo(x, 0)).unwrap();
            screen.execute(PrintStyledContent(border)).unwrap();
            screen.execute(cursor::MoveTo(x, height - 1)).unwrap();
            screen.execute(PrintStyledContent(border)).unwrap();
        }

        // Draw left and right borders
        for y in 0..height {
            screen.execute(cursor::MoveTo(0, y)).unwrap();
            screen.execute(PrintStyledContent(border)).unwrap();
            screen.execute(cursor::MoveTo(width - 1, y)).unwrap();
            screen.execute(PrintStyledContent(border)).unwrap();
        }
    }

    fn draw_status_bar(&self, screen: &mut Screen) {
        let (width, height) = terminal::size().unwrap();
        let formatted = format!(
            " Press ESC to exit | Player 1 Score: {} | Player 2 Score: {} ",
            self.players[0].score, self.players[1].score
        );
        let len = formatted.len();
        screen.execute(SetForegroundColor(Color::White)).unwrap();
        screen
            .execute(cursor::MoveTo((width - len as u16) / 2, height - 1))
            .unwrap();
        print!("{}", formatted);
    }

    fn clear_game_area(&self, screen: &mut Screen) {
        let (_width, height) = terminal::size().unwrap();

        for y in 1..height - 1 {
            execute!(
                screen,
                cursor::MoveTo(1, y),
                terminal::Clear(ClearType::CurrentLine)
            )
            .unwrap();
        }
    }

    fn game_loop(&mut self) {
        let mut screen: Screen = BufWriter::new(stdout());

        if event::poll(Duration::from_millis(40)).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Char('a') => {
                        self.move_player_by(0, -1, 0);
                    }
                    KeyCode::Char('d') => {
                        self.move_player_by(0, 1, 0);
                    }
                    KeyCode::Char('w') => {
                        self.move_player_by(0, 0, -1);
                    }
                    KeyCode::Char('s') => {
                        self.move_player_by(0, 0, 1);
                    }
                    KeyCode::Char(' ') => {
                        self.make_fire(0);
                    }
                    KeyCode::Left => {
                        self.move_player_by(1, -1, 0);
                    }
                    KeyCode::Right => {
                        self.move_player_by(1, 1, 0);
                    }
                    KeyCode::Up => {
                        self.move_player_by(1, 0, -1);
                    }
                    KeyCode::Down => {
                        self.move_player_by(1, 0, 1);
                    }
                    KeyCode::Enter => {
                        self.make_fire(1);
                    }
                    KeyCode::Esc => {
                        self.quit = true;
                    }
                    _ => {}
                }
            }
        }

        // Update sprites (fires etc)
        for fire in &mut self.fires {
            fire.update();
        }

        // Clear screen and draw the player and other sprites
        // stdout
        //    .execute(terminal::Clear(terminal::ClearType::All))
        //   .unwrap();

        self.clear_game_area(&mut screen);

        for player in &mut self.players {
            player.draw(&mut screen);
        }

        for fire in &self.fires {
            fire.draw(&mut screen);
        }

        self.draw_frame(&mut screen);
        self.draw_status_bar(&mut screen);

        screen.flush().unwrap();
    }

    pub fn start(&mut self) {
        let mut screen: Screen = BufWriter::new(stdout());

        // Prepare the terminal
        execute!(screen, terminal::EnterAlternateScreen, cursor::Hide).unwrap();
        terminal::enable_raw_mode().unwrap();

        // Get the terminal size
        let (width, height) = terminal::size().unwrap();
        self.move_player_to(0, width as i32 / 4, height as i32 / 2);
        self.move_player_to(1, width as i32 * 3 / 4, height as i32 / 2);

        screen.flush().unwrap();

        // Play the game
        while !self.quit {
            self.game_loop();
        }

        // Clean up
        execute!(screen, terminal::LeaveAlternateScreen, cursor::Show).unwrap();
        terminal::disable_raw_mode().unwrap();
        screen.flush().unwrap();


        let elapsed = self.start_time.elapsed();
        println!("Game ended! The game lasted {:.2?}.", elapsed);
    }
}
