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

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::default().app_loop(terminal);
    ratatui::restore();
    app_result
}

/// Application state
struct App {
    paddle_1: Vec<(usize, usize)>,
    paddle_2: Vec<(usize, usize)>,
    ball_real_pos: (f32, f32),
    ball_direction: (f32, f32),
    dead: bool,
    // last_move is used to spin the ball to change the direction
    paddle_1_last_move: (Direction, Instant),
    paddle_2_last_move: (Direction, Instant),
    last_move_ball: std::time::Instant,
}

/// Initial app state
impl Default for App {
    fn default() -> Self {
        let paddle_1 = vec![
            (0, 4),
            (0, 5),
            (0, 6),
            (0, 7),
            (0, 8),
            (0, 9),
            (0, 10),
            (0, 11),
            (0, 12),
            (0, 13),
            (0, 14),
            (0, 15),
        ];
        let paddle_2 = vec![
            (19, 4),
            (19, 5),
            (19, 6),
            (19, 7),
            (19, 8),
            (19, 9),
            (19, 10),
            (19, 11),
            (19, 12),
            (19, 13),
            (19, 14),
            (19, 15),
        ];
        let ball_real_pos = (5.0, 12.0);
        let ball_direction = (1.0, 1.0);
        App {
            paddle_1,
            paddle_2,
            ball_real_pos,
            ball_direction,
            dead: false,
            paddle_1_last_move: (Direction::Up, std::time::Instant::now()),
            paddle_2_last_move: (Direction::Up, std::time::Instant::now()),
            last_move_ball: std::time::Instant::now(),
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

        let horizontal = Layout::horizontal([Constraint::Length(62), Constraint::Fill(1)]);
        let [content_area, _extra_horizontal_area] = horizontal.areas(frame_area);

        let vertical = Layout::vertical([
            Constraint::Length(22),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ]);
        let [game_area, instructions_area, dead_area, debug_area, _extra_vertical_ares] = vertical.areas(content_area);
        let ball_pos = (self.ball_real_pos.0.round() as usize, self.ball_real_pos.1.round() as usize);
        let mut text = Text::default();
        for y in 0..20 {
            let mut line = Line::default();
            for x in 0..20 {
                if (x, y) == ball_pos {
                    line.push_span("bal");
                } else if self.paddle_1.contains(&(x, y)) {
                    line.push_span("111");
                } else if self.paddle_2.contains(&(x, y)) {
                    line.push_span("222");
                } else {
                    line.push_span("   ");
                }
            }
            text.push_line(line);
        }

        let game_content = Paragraph::new(text).block(Block::bordered().title("PONG_rat").on_blue());
        frame.render_widget(game_content, game_area);

        let horizontal_instructions = Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)]);
        let [time_area, points_area, exit_area] = horizontal_instructions.areas(instructions_area);

        let paragraph = Paragraph::new("W-up, S-down");
        frame.render_widget(paragraph, time_area);

        let paragraph = Paragraph::new("I-up, K-down");
        frame.render_widget(paragraph, points_area);

        let paragraph = Paragraph::new("Press Q to quit");
        frame.render_widget(paragraph, exit_area);

        if self.dead {
            let paragraph = Paragraph::new("The ball is out! Press N to restart.");
            frame.render_widget(paragraph, dead_area);
        }

        let paragraph = Paragraph::new(format!("direction: {}  {}", self.ball_direction.0, self.ball_direction.1));
        frame.render_widget(paragraph, debug_area);
    }

    fn move_paddle(&mut self, direction: Direction, player: Player) {
        let paddle = match &player {
            Player::One => &mut self.paddle_1,
            Player::Two => &mut self.paddle_2,
        };
        let paddle_last_move = match player {
            Player::One => &mut self.paddle_1_last_move,
            Player::Two => &mut self.paddle_2_last_move,
        };
        match direction {
            Direction::Down => {
                if paddle[2].1 < 19 {
                    for i in 0..paddle.len() {
                        paddle[i].1 += 1;
                    }
                    paddle_last_move.0 = Direction::Down;
                    paddle_last_move.1 = std::time::Instant::now();
                }
            }
            Direction::Up => {
                if paddle[0].1 > 0 {
                    for i in 0..paddle.len() {
                        paddle[i].1 -= 1;
                    }
                    paddle_last_move.0 = Direction::Up;
                    paddle_last_move.1 = std::time::Instant::now();
                }
            }
        }
    }

    fn move_ball(&mut self) {
        if !self.dead {
            // move ball every 300 millis
            let now = std::time::Instant::now();
            if now.duration_since(self.last_move_ball) > Duration::from_millis(300) {
                self.last_move_ball = now;

                self.ball_real_pos.0 += self.ball_direction.0;
                self.ball_real_pos.1 += self.ball_direction.1;
                let ball_pos = (self.ball_real_pos.0.round() as usize, self.ball_real_pos.1.round() as usize);

                // bottom and top have the same bounce angle
                if ball_pos.1 == 19 {
                    self.ball_direction.1 *= -1.0;
                }
                if ball_pos.1 == 0 {
                    self.ball_direction.1 *= -1.0;
                }

                // left and right if bounce from paddle else dead
                if ball_pos.0 == 19 {
                    if self.paddle_2.contains(&ball_pos) {
                        // TODO: to avoid second move in the 19 location
                        self.ball_real_pos.0 = 18.0;
                        // if the paddle has moved in the last 200 ms, then it is a spin
                        // and the angle changes
                        if now.duration_since(self.paddle_2_last_move.1) < Duration::from_millis(200) {
                            match self.paddle_2_last_move.0 {
                                Direction::Up => {
                                    self.ball_direction.0 *= -1.33;
                                    self.ball_direction.1 *= 0.66;
                                }
                                Direction::Down => {
                                    self.ball_direction.0 *= -0.66;
                                    self.ball_direction.1 *= 1.33;
                                }
                            }
                        } else {
                            // standard bounce without spin
                            self.ball_direction.0 *= -1.0;
                        }
                    } else {
                        self.dead = true;
                    }
                }
                if ball_pos.0 == 0 {
                    if self.paddle_1.contains(&ball_pos) {
                        // TODO: to avoid second move in the 0 location
                        self.ball_real_pos.0 = 1.0;
                        // if the paddle has moved in the last 200 ms, then it is a spin
                        // and the angle changes
                        if now.duration_since(self.paddle_1_last_move.1) < Duration::from_millis(200) {
                            match self.paddle_1_last_move.0 {
                                Direction::Up => {
                                    self.ball_direction.0 *= -1.33;
                                    self.ball_direction.1 *= 0.66;
                                }
                                Direction::Down => {
                                    self.ball_direction.0 *= -0.66;
                                    self.ball_direction.1 *= 1.33;
                                }
                            }
                        } else {
                            // standard bounce without spin
                            self.ball_direction.0 *= -1.0;
                        }
                    } else {
                        self.dead = true;
                    }
                }
            }
        }
    }

    fn restart_game(&mut self) {
        *self = Self::default();
    }
}
