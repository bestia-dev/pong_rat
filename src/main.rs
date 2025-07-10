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

const WIDTH_LEN: usize = 90;
const WIDTH_LAST_ELEMENT: usize = WIDTH_LEN - 1;
const HEIGHT_LEN: usize = 26;
const HEIGHT_LAST_ELEMENT: usize = HEIGHT_LEN - 1;
const PADDLE_LEN_INIT: usize = 10;
const MID_WIDTH: usize = WIDTH_LEN / 2;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let app_result = App::init().app_loop(terminal);

    ratatui::restore();
    app_result
}

/// Data inside one game
struct GameData {
    paddle_1: Vec<(usize, usize)>,
    paddle_2: Vec<(usize, usize)>,
    ball_real_pos: (f32, f32),
    ball_direction: (f32, f32),
    // last_move is used to spin the ball to change the direction
    paddle_1_last_move: (Direction, Instant),
    paddle_2_last_move: (Direction, Instant),
    last_move_ball: std::time::Instant,
    dead: bool,
}

/// Application state
struct App {
    game_data: GameData,
    net: [(usize, usize); HEIGHT_LEN],
    ball_step_millisecond: Duration,
    paddle_len: usize,
    player_1_points: u32,
    player_2_points: u32,
}

/// Initial state for every game
impl GameData {
    fn init(paddle_len: usize) -> Self {
        let mut paddle_1 = vec![];
        for i in 0..paddle_len {
            paddle_1.push((0, i + 4));
        }

        let mut paddle_2 = vec![];
        for i in 0..paddle_len {
            paddle_2.push((WIDTH_LAST_ELEMENT, i + 4));
        }

        let mut net: [(usize, usize); HEIGHT_LEN] = [(0, 0); HEIGHT_LEN];
        for (i, elem) in net.iter_mut().enumerate() {
            *elem = (MID_WIDTH, i);
        }

        let ball_real_pos = (WIDTH_LEN as f32 / 2.0, HEIGHT_LEN as f32 / 2.0);
        let ball_direction = (1.0, 1.0);
        GameData {
            paddle_1,
            paddle_2,
            ball_real_pos,
            ball_direction,
            paddle_1_last_move: (Direction::Up, std::time::Instant::now()),
            paddle_2_last_move: (Direction::Up, std::time::Instant::now()),
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
            player_1_points: 0,
            player_2_points: 0,
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
        let horizontal = Layout::horizontal([Constraint::Length((WIDTH_LEN as u16) + 2), Constraint::Fill(1)]);
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
        let ball_pos = (
            self.game_data.ball_real_pos.0.round() as usize,
            self.game_data.ball_real_pos.1.round() as usize,
        );
        let mut text = Text::default();
        for y in 0..HEIGHT_LEN {
            let mut line = Line::default();
            for x in 0..WIDTH_LEN {
                if (x, y) == ball_pos {
                    line.push_span("0");
                } else if x == 0 && self.game_data.paddle_1.contains(&(x, y)) {
                    line.push_span("1");
                } else if x == WIDTH_LAST_ELEMENT && self.game_data.paddle_2.contains(&(x, y)) {
                    line.push_span("2");
                } else if x == MID_WIDTH && self.net.contains(&(x, y)) {
                    line.push_span("|");
                } else {
                    line.push_span(" ");
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

        let paragraph = Paragraph::new(format!("{} : {}", self.player_1_points, self.player_2_points)).centered();
        frame.render_widget(paragraph, points_area);

        //let paragraph = Paragraph::new(format!("direction: {}  {}", self.game_data.ball_direction.0, self.ball_direction.1));
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
                self.game_data.paddle_1.push((
                    self.game_data.paddle_1.last().unwrap().0,
                    self.game_data.paddle_1.last().unwrap().1 + 1,
                ));
                self.game_data.paddle_2.push((
                    self.game_data.paddle_2.last().unwrap().0,
                    self.game_data.paddle_2.last().unwrap().1 + 1,
                ));
            }
            Direction::Down => {
                self.paddle_len -= 1;
                self.game_data.paddle_1.pop().unwrap();
                self.game_data.paddle_2.pop().unwrap();
            }
        }
    }

    fn move_paddle(&mut self, direction: Direction, player: Player) {
        let paddle = match &player {
            Player::One => &mut self.game_data.paddle_1,
            Player::Two => &mut self.game_data.paddle_2,
        };
        let paddle_last_move = match player {
            Player::One => &mut self.game_data.paddle_1_last_move,
            Player::Two => &mut self.game_data.paddle_2_last_move,
        };
        match direction {
            Direction::Down => {
                if paddle[self.paddle_len - 1].1 < HEIGHT_LAST_ELEMENT {
                    for elem in paddle.iter_mut() {
                        elem.1 += 1;
                    }
                    paddle_last_move.0 = Direction::Down;
                    paddle_last_move.1 = std::time::Instant::now();
                }
            }
            Direction::Up => {
                if paddle[0].1 > 0 {
                    for elem in paddle.iter_mut() {
                        elem.1 -= 1;
                    }
                    paddle_last_move.0 = Direction::Up;
                    paddle_last_move.1 = std::time::Instant::now();
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

                self.game_data.ball_real_pos.0 += self.game_data.ball_direction.0;
                self.game_data.ball_real_pos.1 += self.game_data.ball_direction.1;
                let ball_pos = (
                    self.game_data.ball_real_pos.0.round() as usize,
                    self.game_data.ball_real_pos.1.round() as usize,
                );

                // bottom and top have the same bounce angle
                if ball_pos.1 == HEIGHT_LAST_ELEMENT {
                    self.game_data.ball_direction.1 *= -1.0;
                }
                if ball_pos.1 == 0 {
                    self.game_data.ball_direction.1 *= -1.0;
                }

                // left and right if bounce from paddle else dead
                if ball_pos.0 == WIDTH_LAST_ELEMENT {
                    if self.game_data.paddle_2.contains(&ball_pos) {
                        // Force move it out of the paddle. The next x move can be less then 1
                        // and this means the ball would be inside the paddle again. That is not good.
                        self.game_data.ball_real_pos.0 = WIDTH_LAST_ELEMENT as f32 - 1.0;
                        // if the paddle has moved in the last 200 ms, then it is a spin
                        // and the angle changes
                        if now.duration_since(self.game_data.paddle_2_last_move.1) < Duration::from_millis(200) {
                            match self.game_data.paddle_2_last_move.0 {
                                Direction::Up => {
                                    self.game_data.ball_direction.0 *= -1.2;
                                    self.game_data.ball_direction.1 *= 0.8;
                                }
                                Direction::Down => {
                                    self.game_data.ball_direction.0 *= -0.8;
                                    self.game_data.ball_direction.1 *= 1.2;
                                }
                            }
                        } else {
                            // standard bounce without spin
                            self.game_data.ball_direction.0 *= -1.0;
                        }
                    } else {
                        self.player_1_points += 1;
                        self.game_data.dead = true;
                    }
                }
                if ball_pos.0 == 0 {
                    if self.game_data.paddle_1.contains(&ball_pos) {
                        // Force move it out of the paddle. The next x move can be less then 1
                        // and this means the ball would be inside the paddle again. That is not good.
                        self.game_data.ball_real_pos.0 = 1.0;
                        // if the paddle has moved in the last 200 ms, then it is a spin
                        // and the angle changes
                        if now.duration_since(self.game_data.paddle_1_last_move.1) < Duration::from_millis(200) {
                            match self.game_data.paddle_1_last_move.0 {
                                Direction::Up => {
                                    self.game_data.ball_direction.0 *= -1.2;
                                    self.game_data.ball_direction.1 *= 0.8;
                                }
                                Direction::Down => {
                                    self.game_data.ball_direction.0 *= -0.8;
                                    self.game_data.ball_direction.1 *= 1.2;
                                }
                            }
                            // Limit to min and max angle for direction
                            if self.game_data.ball_direction.0.abs() > 1.8 {
                                self.game_data.ball_direction.0 = self.game_data.ball_direction.0.signum() * 1.8;
                            }
                            if self.game_data.ball_direction.0 < 0.3 {
                                self.game_data.ball_direction.0 = self.game_data.ball_direction.0.signum() * 0.3;
                            }
                            if self.game_data.ball_direction.1 > 1.8 {
                                self.game_data.ball_direction.1 = self.game_data.ball_direction.1.signum() * 1.8;
                            }
                            if self.game_data.ball_direction.1 < 0.3 {
                                self.game_data.ball_direction.1 = self.game_data.ball_direction.1.signum() * 0.3;
                            }
                        } else {
                            // standard bounce without spin
                            self.game_data.ball_direction.0 *= -1.0;
                        }
                    } else {
                        self.player_2_points += 1;
                        self.game_data.dead = true;
                    }
                }
            }
        }
    }

    fn restart_game(&mut self) {
        // leave paddle in the same position
        let paddle_1 = self.game_data.paddle_1.clone();
        let paddle_2 = self.game_data.paddle_2.clone();
        self.game_data = GameData::init(self.paddle_len);
        self.game_data.paddle_1 = paddle_1;
        self.game_data.paddle_2 = paddle_2;
    }
}
