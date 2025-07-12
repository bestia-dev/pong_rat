//! src/bin/pong_rat/main.rs

use std::time::{Duration, Instant};

use color_eyre::Result;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, poll},
    layout::{Constraint, Layout},
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Paragraph},
};

const WIDTH_LEN: usize = 45;
const WIDTH_LAST_ELEMENT: usize = WIDTH_LEN - 1;
const HEIGHT_LEN: usize = 26;
const HEIGHT_LAST_ELEMENT: usize = HEIGHT_LEN - 1;
const PADDLE_LEN_INIT: usize = 10;
const MID_WIDTH: usize = WIDTH_LEN / 2;
const SPRITE_WIDTH: u16 = 2;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let app_result = App::init().app_loop(terminal);

    ratatui::restore();
    app_result
}

#[derive(PartialEq, Clone)]
struct IPos {
    x: usize,
    y: usize,
}

struct FPos {
    x: f32,
    y: f32,
}

struct LastMove {
    direction: Direction,
    instant: Instant,
}

/// Data inside one game
struct GameData {
    paddle_left: Vec<IPos>,
    paddle_right: Vec<IPos>,
    ball_real_pos: FPos,
    ball_direction: FPos,
    // last_move is used to spin the ball to change the direction
    paddle_left_last_move: LastMove,
    paddle_right_last_move: LastMove,
    last_move_ball: std::time::Instant,
    dead: bool,
}

/// Application state
struct App {
    game_data: GameData,
    net: [(usize, usize); HEIGHT_LEN],
    ball_step_millisecond: Duration,
    paddle_len: usize,
    player_left_points: u32,
    player_right_points: u32,
}

/// Initial state for every game
impl GameData {
    fn init(paddle_len: usize) -> Self {
        let mut paddle_left = vec![];
        let x = 0;
        for i in 0..paddle_len {
            let y = i + 4;
            paddle_left.push(IPos { x, y });
        }

        let mut paddle_right = vec![];
        let x = WIDTH_LAST_ELEMENT;
        for i in 0..paddle_len {
            let y = i + 4;
            paddle_right.push(IPos { x, y });
        }

        let mut net: [(usize, usize); HEIGHT_LEN] = [(0, 0); HEIGHT_LEN];
        for (i, elem) in net.iter_mut().enumerate() {
            *elem = (MID_WIDTH, i);
        }

        let ball_real_pos = FPos {
            x: WIDTH_LEN as f32 / 2.0,
            y: HEIGHT_LEN as f32 / 2.0,
        };
        let ball_direction = FPos { x: 1.0, y: 1.0 };
        GameData {
            paddle_left,
            paddle_right,
            ball_real_pos,
            ball_direction,
            paddle_left_last_move: LastMove {
                direction: Direction::Up,
                instant: std::time::Instant::now(),
            },
            paddle_right_last_move: LastMove {
                direction: Direction::Up,
                instant: std::time::Instant::now(),
            },
            last_move_ball: std::time::Instant::now(),
            dead: false,
        }
    }
}

/// Initial app state
impl App {
    fn init() -> Self {
        let mut net: [(usize, usize); HEIGHT_LEN] = [(0, 0); HEIGHT_LEN];
        for (i, elem) in net.iter_mut().enumerate() {
            *elem = (MID_WIDTH, i);
        }
        let paddle_len = PADDLE_LEN_INIT;
        App {
            game_data: GameData::init(paddle_len),
            net,
            ball_step_millisecond: Duration::from_millis(200),
            paddle_len,
            player_left_points: 0,
            player_right_points: 0,
        }
    }
}

pub enum Direction {
    Up,
    Down,
}

pub enum Player {
    One,
    Two,
}

impl App {
    /// Ratatui app is a loop with 2 functions: draw and react events
    fn app_loop(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if poll(std::time::Duration::from_millis(0))? {
                // It's guaranteed that `read` won't block, because `poll` returned
                // `Ok(true)`.
                // react on events
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                            KeyCode::Char('q') => return Ok(()),

                            KeyCode::Char('n') => self.restart_game(),

                            KeyCode::Char('w') => self.move_paddle(Direction::Up, Player::One),
                            KeyCode::Char('s') => self.move_paddle(Direction::Down, Player::One),

                            KeyCode::Char('i') => self.move_paddle(Direction::Up, Player::Two),
                            KeyCode::Char('k') => self.move_paddle(Direction::Down, Player::Two),

                            KeyCode::Char('9') => self.ball_speed(Direction::Down),
                            KeyCode::Char('0') => self.ball_speed(Direction::Up),

                            KeyCode::Char('1') => self.paddle_length(Direction::Down),
                            KeyCode::Char('2') => self.paddle_length(Direction::Up),
                            _ => {}
                        }
                    }
                    self.move_ball();
                }
            } else {
                // Timeout expired, no `Event` is available
                self.move_ball();
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let frame_area = frame.area();

        // add 2 for the block border
        let horizontal = Layout::horizontal([Constraint::Length((WIDTH_LEN as u16) * SPRITE_WIDTH + 2), Constraint::Fill(1)]);
        let [content_area, _extra_horizontal_area] = horizontal.areas(frame_area);

        let vertical = Layout::vertical([
            // add 2 for the block border
            Constraint::Length(HEIGHT_LEN as u16 + 2),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ]);
        let [
            game_area,
            points_area,
            instructions_area,
            dead_area,
            _debug_area,
            _extra_vertical_ares,
        ] = vertical.areas(content_area);

        let ball_pos = IPos {
            x: self.game_data.ball_real_pos.x.round() as usize,
            y: self.game_data.ball_real_pos.y.round() as usize,
        };
        let mut text = Text::default();
        for y in 0..HEIGHT_LEN {
            let mut line = Line::default();
            for x in 0..WIDTH_LEN {
                if (IPos { x, y }) == ball_pos {
                    line.push_span("██");
                } else if x == 0 && self.game_data.paddle_left.contains(&IPos { x, y }) {
                    line.push_span(" █");
                } else if x == WIDTH_LAST_ELEMENT && self.game_data.paddle_right.contains(&IPos { x, y }) {
                    line.push_span("█ ");
                } else if x == MID_WIDTH && self.net.contains(&(x, y)) {
                    line.push_span("| ");
                } else {
                    line.push_span("  ");
                }
            }
            text.push_line(line);
        }

        let game_content = Paragraph::new(text).block(Block::bordered().title("PONG_rat").on_blue());
        frame.render_widget(game_content, game_area);

        let horizontal_instructions = Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)]);
        let [instr_left_area, instr_center_area, instr_right_area] = horizontal_instructions.areas(instructions_area);

        let paragraph = Paragraph::new("1-smaller, 2-larger, 9-slower, 0-faster");
        frame.render_widget(paragraph, instr_left_area);

        let paragraph = Paragraph::new("W-up, S-down, I-up, K-down").centered();
        frame.render_widget(paragraph, instr_center_area);

        let paragraph = Paragraph::new("Press Q to quit").right_aligned();
        frame.render_widget(paragraph, instr_right_area);

        if self.game_data.dead {
            let paragraph = Paragraph::new("The ball is out! Press N to restart.");
            frame.render_widget(paragraph, dead_area);
        }

        let paragraph = Paragraph::new(format!("{} : {}", self.player_left_points, self.player_right_points)).centered();
        frame.render_widget(paragraph, points_area);

        //let paragraph = Paragraph::new(format!("direction: {}  {}", self.game_data.ball_direction.x, self.ball_direction.y));
        //frame.render_widget(paragraph, debug_area);
    }

    fn ball_speed(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.ball_step_millisecond = self.ball_step_millisecond.checked_sub(Duration::from_millis(20)).unwrap(),
            Direction::Down => self.ball_step_millisecond = self.ball_step_millisecond.checked_add(Duration::from_millis(20)).unwrap(),
        }
    }

    fn paddle_length(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                self.paddle_len += 1;
                let x = self.game_data.paddle_left.last().unwrap().x;
                let y = self.game_data.paddle_left.last().unwrap().y + 1;
                self.game_data.paddle_left.push(IPos { x, y });
                let x = self.game_data.paddle_right.last().unwrap().x;
                let y = self.game_data.paddle_right.last().unwrap().y + 1;
                self.game_data.paddle_right.push(IPos { x, y });
            }
            Direction::Down => {
                self.paddle_len -= 1;
                self.game_data.paddle_left.pop().unwrap();
                self.game_data.paddle_right.pop().unwrap();
            }
        }
    }

    fn move_paddle(&mut self, direction: Direction, player: Player) {
        let paddle = match &player {
            Player::One => &mut self.game_data.paddle_left,
            Player::Two => &mut self.game_data.paddle_right,
        };
        let paddle_last_move = match player {
            Player::One => &mut self.game_data.paddle_left_last_move,
            Player::Two => &mut self.game_data.paddle_right_last_move,
        };
        paddle_last_move.instant = std::time::Instant::now();
        match direction {
            Direction::Down => {
                paddle_last_move.direction = Direction::Down;
                if paddle[self.paddle_len - 1].y < HEIGHT_LAST_ELEMENT {
                    for elem in paddle.iter_mut() {
                        elem.y += 1;
                    }
                }
            }
            Direction::Up => {
                paddle_last_move.direction = Direction::Up;
                if paddle[0].y > 0 {
                    for elem in paddle.iter_mut() {
                        elem.y -= 1;
                    }
                }
            }
        }
    }

    fn move_ball(&mut self) {
        if !self.game_data.dead {
            // move ball every ball_step_millisecond
            let now = std::time::Instant::now();
            if now.duration_since(self.game_data.last_move_ball) > self.ball_step_millisecond {
                self.game_data.last_move_ball = now;

                self.game_data.ball_real_pos.x += self.game_data.ball_direction.x;
                self.game_data.ball_real_pos.y += self.game_data.ball_direction.y;
                let ball_pos = IPos {
                    x: self.game_data.ball_real_pos.x.round() as usize,
                    y: self.game_data.ball_real_pos.y.round() as usize,
                };

                // bottom and top have the same bounce angle
                if ball_pos.y == HEIGHT_LAST_ELEMENT {
                    self.game_data.ball_direction.y *= -1.0;
                }
                if ball_pos.y == 0 {
                    self.game_data.ball_direction.y *= -1.0;
                }

                // left and right if bounce from paddle else dead
                if ball_pos.x == WIDTH_LAST_ELEMENT {
                    if self.game_data.paddle_right.contains(&ball_pos) {
                        // Force move it out of the paddle. The next x move can be less then 1
                        // and this means the ball would be inside the paddle again. That is not good.
                        self.game_data.ball_real_pos.x = WIDTH_LAST_ELEMENT as f32 - 1.0;
                        // if the paddle has moved in the last 200 ms, then it is a spin
                        // and the angle changes
                        if now.duration_since(self.game_data.paddle_right_last_move.instant) < Duration::from_millis(200) {
                            match self.game_data.paddle_right_last_move.direction {
                                Direction::Up => {
                                    self.game_data.ball_direction.x *= -1.2;
                                    self.game_data.ball_direction.y *= 0.8;
                                }
                                Direction::Down => {
                                    self.game_data.ball_direction.x *= -0.8;
                                    self.game_data.ball_direction.y *= 1.2;
                                }
                            }
                        } else {
                            // standard bounce without spin
                            self.game_data.ball_direction.x *= -1.0;
                        }
                    } else {
                        self.player_left_points += 1;
                        self.game_data.dead = true;
                    }
                }
                if ball_pos.x == 0 {
                    if self.game_data.paddle_left.contains(&ball_pos) {
                        // Force move it out of the paddle. The next x move can be less then 1
                        // and this means the ball would be inside the paddle again. That is not good.
                        self.game_data.ball_real_pos.x = 1.0;
                        // if the paddle has moved in the last 200 ms, then it is a spin
                        // and the angle changes
                        if now.duration_since(self.game_data.paddle_left_last_move.instant) < Duration::from_millis(200) {
                            match self.game_data.paddle_left_last_move.direction {
                                Direction::Up => {
                                    self.game_data.ball_direction.x *= -1.2;
                                    self.game_data.ball_direction.y *= 0.8;
                                }
                                Direction::Down => {
                                    self.game_data.ball_direction.x *= -0.8;
                                    self.game_data.ball_direction.y *= 1.2;
                                }
                            }
                            // Limit to min and max angle for direction
                            if self.game_data.ball_direction.x.abs() > 1.8 {
                                self.game_data.ball_direction.x = self.game_data.ball_direction.x.signum() * 1.8;
                            }
                            if self.game_data.ball_direction.x < 0.3 {
                                self.game_data.ball_direction.x = self.game_data.ball_direction.x.signum() * 0.3;
                            }
                            if self.game_data.ball_direction.y > 1.8 {
                                self.game_data.ball_direction.y = self.game_data.ball_direction.y.signum() * 1.8;
                            }
                            if self.game_data.ball_direction.y < 0.3 {
                                self.game_data.ball_direction.y = self.game_data.ball_direction.y.signum() * 0.3;
                            }
                        } else {
                            // standard bounce without spin
                            self.game_data.ball_direction.x *= -1.0;
                        }
                    } else {
                        self.player_right_points += 1;
                        self.game_data.dead = true;
                    }
                }
            }
        }
    }

    fn restart_game(&mut self) {
        // leave paddle in the same position
        let paddle_1 = self.game_data.paddle_left.clone();
        let paddle_2 = self.game_data.paddle_right.clone();
        self.game_data = GameData::init(self.paddle_len);
        self.game_data.paddle_left = paddle_1;
        self.game_data.paddle_right = paddle_2;
    }
}
