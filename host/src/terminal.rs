use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph, BorderType},
    layout::{Layout, Direction, Constraint, Rect, Alignment},
    Terminal,
    style::{Style, Color},
    text::{Span, Line, Text}
};
use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode},
    event::{self, Event, KeyCode, KeyModifiers},
};

use std::io::stdout;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};



pub struct VerificationResult {
    pub is_hit: bool,
    pub proof_valid: bool,
    pub commitment: String,
    pub ship_sunk: Option<String>,
}

const SHIPS: [(&str, usize); 5] = [
    ("Carrier", 5),
    ("Battleship", 4),
    ("Cruiser", 3),
    ("Submarine", 3),
    ("Destroyer", 2),
];



fn draw_grid(board: &[[char; 10]; 10]) -> Text<'static> {
    let mut lines = vec![];

    // header
    lines.push(Line::from(" 1   2   3   4   5   6   7   8   9   10"));

    for row in 0..10 {
        let mut spans = vec![];
        
        // row number
        spans.push(Span::raw(format!("{} ", row + 1)));
        spans.push(Span::raw(" "));

        for col in 0..10 {
            let cell = board[row][col];
            let span = match cell {
                'S' => Span::styled("S   ", Style::default().fg(Color::Blue)),
                'X' => Span::styled("X   ", Style::default().fg(Color::Red)),
                'O' => Span::styled("O   ", Style::default().fg(Color::Green)),
                _   => Span::raw(".   "),
            };
            spans.push(span);
        }

        lines.push(Line::from(spans));
        lines.push(Line::from(""));  
    }

    Text::from(lines)
}


fn place_ship(
    board: &mut [[char; 10]; 10],
    input: &str,
    size: usize,
) -> Result<(), String> {
    let parts: Vec<&str> = input.trim().split(',').collect();

    if parts.len() != 4 {
        return Err("format must be: row_start,col_start,row_end,col_end".to_string());
    }

    let parsed: Vec<usize> = parts.iter()
        .map(|p| p.trim().parse::<usize>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "all values must be numbers".to_string())?;

    let (r1, c1, r2, c2) = (parsed[0]-1, parsed[1]-1, parsed[2]-1, parsed[3]-1);

    // check bounds
    if r1 >= 10 || c1 >= 10 || r2 >= 10 || c2 >= 10 {
        return Err("coordinates must be between 0 and 9".to_string());
    }

    // check straight line
    if r1 != r2 && c1 != c2 {
        return Err("ship must be placed horizontally or vertically".to_string());
    }

    // check size
    let ship_size = if r1 == r2 {
        (c2 as isize - c1 as isize).unsigned_abs() + 1
    } else {
        (r2 as isize - r1 as isize).unsigned_abs() + 1
    };

    if ship_size != size {
        return Err(format!("ship size must be {}, you entered {}", size, ship_size));
    }

    // collect cells
    let cells: Vec<(usize, usize)> = if r1 == r2 {
        let (start, end) = if c1 < c2 { (c1, c2) } else { (c2, c1) };
        (start..=end).map(|c| (r1, c)).collect()
    } else {
        let (start, end) = if r1 < r2 { (r1, r2) } else { (r2, r1) };
        (start..=end).map(|r| (r, c1)).collect()
    };

    // check overlap
    for &(r, c) in &cells {
        if board[r][c] == 'S' {
            return Err("ship overlaps with another ship".to_string());
        }
    }

    // place ship
    for (r, c) in cells {
        board[r][c] = 'S';
    }

    Ok(())
}


fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}


fn create_layout(
    direction: Direction,
    constraints: Vec<Constraint>,
    area: Rect,
) -> Vec<Rect> {
    Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area)
        .to_vec()
}

pub fn setup_player_terminal(player: usize) -> ([[char; 10]; 10], String) {
    execute!(stdout(), Clear(ClearType::All), Clear(ClearType::Purge)).unwrap();
    enable_raw_mode().unwrap();

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).unwrap();
    
    let mut head_msg = format!("ZK-Battleship-Game\nPlayer {} - placing ships stage", player);
    let mut status_msg = format!("Welcome Player {}! Enter your salt first!", player);
    let mut input = String::new();
    let mut board = [['.' ; 10]; 10];
    let mut salt = String::new();
    let mut grid_text = draw_grid(&board);


    let mut stage = 0;
    let mut ship_index = 0;

    loop {
        terminal.draw(|frame| {
            let chunks = create_layout(Direction::Vertical, vec![
                Constraint::Percentage(8),
                Constraint::Percentage(78),
                Constraint::Percentage(7),
                Constraint::Percentage(7),
            ], frame.size());

            let outer_block = Block::default()
                .title(format!("Player {} Board", player))
                .borders(Borders::ALL)
                .border_type(BorderType::Double);

            let inner = outer_block.inner(chunks[1]);
            frame.render_widget(outer_block, chunks[1]);

            let area = centered_rect(40, 70, inner);

            let head = Paragraph::new(head_msg.clone())
                .alignment(Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double));

            let board_widget = Paragraph::new(grid_text.clone())
                .alignment(Alignment::Center)
                .block(Block::default()
                    .title("grid")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double));

            let message = Paragraph::new(status_msg.clone())
                .block(Block::default()
                    .title("messages")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double));

            let inputs = Paragraph::new(input.clone())
                .block(Block::default()
                    .title(" Write here ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double));

            frame.render_widget(head, chunks[0]);
            frame.render_widget(board_widget, area);
            frame.render_widget(message, chunks[2]);
            frame.render_widget(inputs, chunks[3]);
        });

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Char(c) => input.push(c),
                KeyCode::Backspace => { input.pop(); }
                KeyCode::Enter => {
                    match stage {
                        // enter salt
                        0 => {
                            if input.trim().is_empty() {
                                status_msg = "salt cannot be empty!".to_string();
                            } else {
                                salt = input.clone();
                                ship_index = 0;
                                let (name, size) = SHIPS[ship_index];
                                status_msg = format!(
                                    "Place your {} (size {}). format: row_start,col_start,row_end,col_end",
                                    name, size
                                );
                                stage = 1;
                            }
                            input.clear();
                        }
                        // place ships
                        1 => {
                            let (name, size) = SHIPS[ship_index];
                            match place_ship(&mut board, &input, size) {
                                Ok(_) => {
                                    grid_text = draw_grid(&board);
                                    ship_index += 1;
                                    if ship_index >= SHIPS.len() {
                                        input.clear();
                                        break;
                                    } else {
                                        let (next_name, next_size) = SHIPS[ship_index];
                                        status_msg = format!(
                                            "Place your {} (size {}). format: row_start,col_start,row_end,col_end",
                                            next_name, next_size
                                        );
                                    }
                                }
                                Err(e) => {
                                    status_msg = format!("Error: {}. Try again: {} (size {})", e, name, size);
                                }
                            }
                            input.clear();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode().unwrap();

    (board, salt)
}




pub fn rounds_terminal(
    player: usize,
    br: &[[[char; 10]; 10]; 2],
    hashes: &[&String; 2],
    mode: u8,
    verification: Option<&VerificationResult>,
    round: u32,
) -> [usize; 2] {
    execute!(stdout(), Clear(ClearType::All), Clear(ClearType::Purge)).unwrap();
    enable_raw_mode().unwrap();

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).unwrap();
    
    let mut head_msg = format!("ZK-Battleship-Game\nPlayer {} - playing stage Round {}", player, round);
    let mut status_msg = if mode == 0 {
        format!("Player {}: enter your guess! format: row,col", player)
    } else {
        "Press Enter to continue...".to_string()
    };
    let mut input = String::new();
    let board1 = br[0];
    let board2 = br[1];
    let grid1 = draw_grid(&board1);
    let grid2 = draw_grid(&board2);

    loop {
        terminal.draw(|frame| {
            let chunks = if mode == 1 {
                create_layout(Direction::Vertical, vec![
                    Constraint::Percentage(8),
                    Constraint::Percentage(70),
                    Constraint::Percentage(15),
                    Constraint::Percentage(7),
                ], frame.size())
            } else {
                create_layout(Direction::Vertical, vec![
                    Constraint::Percentage(8),
                    Constraint::Percentage(78),
                    Constraint::Percentage(7),
                    Constraint::Percentage(7),
                ], frame.size())
            };

            let areas = create_layout(Direction::Horizontal, vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ], chunks[1]);

            let outer_block1 = Block::default()
                .title(format!("  Player 1 board - commitment: {}  ", hashes[0]))
                .borders(Borders::ALL)
                .border_type(BorderType::Double);

            let outer_block2 = Block::default()
                .title(format!("  Player 2 board - commitment: {}  ", hashes[1]))
                .borders(Borders::ALL)
                .border_type(BorderType::Double);

            let inner1 = outer_block1.inner(areas[0]);
            let inner2 = outer_block2.inner(areas[1]);

            frame.render_widget(outer_block1, areas[0]);
            frame.render_widget(outer_block2, areas[1]);

            let area1 = centered_rect(80, 80, inner1);
            let area2 = centered_rect(80, 80, inner2);

            let head = Paragraph::new(head_msg.clone())
                .alignment(Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double));

            let board_1 = Paragraph::new(grid1.clone())
                .alignment(Alignment::Center)
                .block(Block::default()
                    .title(" Player 1 grid ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double));

            let board_2 = Paragraph::new(grid2.clone())
                .alignment(Alignment::Center)
                .block(Block::default()
                    .title(" Player 2 grid ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double));

            let message = Paragraph::new(status_msg.clone())
                .block(Block::default()
                    .title("messages")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double));

            frame.render_widget(head, chunks[0]);
            frame.render_widget(board_1, area1);
            frame.render_widget(board_2, area2);

            if mode == 1 {
                let v = verification.unwrap();
                let result_color = if v.is_hit { Color::Red } else { Color::Green };
                let proof_color = if v.proof_valid { Color::Green } else { Color::Red };

                let verify_text = Text::from(vec![
                    Line::from(vec![
                        Span::raw("  Result:      "),
                        Span::styled(
                            if v.is_hit { "HIT" } else { "MISS" },
                            Style::default().fg(result_color),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("  Proof:       "),
                        Span::styled(
                            if v.proof_valid { "VALID" } else { "INVALID" },
                            Style::default().fg(proof_color),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("  Commitment: "),
                        Span::styled(v.commitment.clone(), Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(vec![
                        Span::raw("  Ship Sunk: "),
                        match &v.ship_sunk {
                            Some(ship) => Span::styled(
                                format!(" {} SUNK!", ship),
                                Style::default().fg(Color::Red),
                            ),
                            None => Span::styled("None", Style::default().fg(Color::Gray)),
                        },
                    ]),
                ]);

                let verify_widget = Paragraph::new(verify_text)
                    .block(Block::default()
                        .title(" Proof Verification ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Double));

                frame.render_widget(verify_widget, chunks[2]);
                frame.render_widget(message, chunks[3]);

            } else {
                let inputs = Paragraph::new(input.clone())
                    .block(Block::default()
                        .title(" Write here ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Double));

                frame.render_widget(message, chunks[2]);
                frame.render_widget(inputs, chunks[3]);
            }
        });

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Char(c) => {
                    if mode == 0 { input.push(c); }
                }
                KeyCode::Backspace => {
                    if mode == 0 { input.pop(); }
                }
                KeyCode::Enter => {
                    if mode == 1 {
                        disable_raw_mode().unwrap();
                        return [0, 0];
                    }
                    let parts: Vec<&str> = input.trim().split(',').collect();
                    if parts.len() == 2 {
                        if let (Ok(row), Ok(col)) = (parts[0].trim().parse::<usize>(), parts[1].trim().parse::<usize>()) {
                            if row >= 1 && row <= 10 && col >= 1 && col <= 10 {
                                input.clear();
                                disable_raw_mode().unwrap();
                                return [row - 1, col - 1];
                            } else {
                                status_msg = "coordinates must be between 1 and 10".to_string();
                            }
                        } else {
                            status_msg = "invalid input! format: row,col".to_string();
                        }
                    } else {
                        status_msg = "invalid input! format: row,col".to_string();
                    }
                    input.clear();
                }
                _ => {}
            }
        }
    }

    disable_raw_mode().unwrap();
    [0, 0]
}





pub fn game_over_terminal(
    winner: usize,
    salts: &[String; 2],
    hashes: &[String; 2],
) {
    execute!(stdout(), Clear(ClearType::All), Clear(ClearType::Purge)).unwrap();
    enable_raw_mode().unwrap();

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    loop {
        terminal.draw(|frame| {
            let chunks = create_layout(Direction::Vertical, vec![
                Constraint::Percentage(100),
            ], frame.size());

            let outer_block = Block::default()
                .title(" Game Over ")
                .borders(Borders::ALL)
                .border_type(BorderType::Double);

            let inner = outer_block.inner(chunks[0]);
            frame.render_widget(outer_block, chunks[0]);

            let area = centered_rect(60, 70, inner);

            let winner_color = if winner == 1 { Color::Green } else { Color::Cyan };

            let text = Text::from(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        format!("  Player {}  WINS!  ", winner),
                        Style::default().fg(winner_color),
                    ),
                ]),
                Line::from(""),
                Line::from("─────────────────────────────────────"),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  Player 1 salt:        "),
                    Span::styled(salts[0].clone(), Style::default().fg(Color::Yellow)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  Player 2 salt:        "),
                    Span::styled(salts[1].clone(), Style::default().fg(Color::Yellow)),
                ]),
                Line::from(""),
                Line::from("─────────────────────────────────────"),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  Player 1 commitment:  "),
                    Span::styled(hashes[0].clone(), Style::default().fg(Color::Magenta)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  Player 2 commitment:  "),
                    Span::styled(hashes[1].clone(), Style::default().fg(Color::Magenta)),
                ]),
                Line::from(""),
                Line::from("─────────────────────────────────────"),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "  Anyone can verify: SHA256(board + salt) == commitment",
                        Style::default().fg(Color::Gray),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "  Press Enter to exit",
                        Style::default().fg(Color::White),
                    ),
                ]),
            ]);

            let paragraph = Paragraph::new(text)
                .alignment(Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double));

            frame.render_widget(paragraph, area);
        });

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Enter => break,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                _ => {}
            }
        }
    }

    disable_raw_mode().unwrap();
}




