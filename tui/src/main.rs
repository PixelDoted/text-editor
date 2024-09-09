use core::{
    cursor::{Cursor, MoveCursor},
    view::TextView,
};

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    widgets::Paragraph,
};

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    ratatui::crossterm::execute!(
        std::io::stdout(),
        ratatui::crossterm::cursor::SetCursorStyle::SteadyBar
    )
    .unwrap();

    let mut view = TextView::from_str("The quick brown fox jumped over the lazy dog");
    let mut offset = Cursor::default();

    loop {
        terminal.draw(|frame| {
            if offset.col > 0 && view.cursor.col < offset.col + 5 {
                offset.col = view.cursor.col - 5;
            } else if view.cursor.col - offset.col > frame.area().width as usize - 5 {
                offset.col = view.cursor.col + 5 - frame.area().width as usize;
            }

            if offset.row > 0 && view.cursor.row < offset.row + 2 {
                offset.row = view.cursor.row - 2;
            } else if view.cursor.row - offset.row > frame.area().height as usize - 2 {
                offset.row = view.cursor.row + 2 - frame.area().height as usize;
            }

            let mut text_buffer = String::new();
            for row in offset.row..offset.row + frame.area().height as usize {
                let Some(chars) = view
                    .buffer
                    .get_line(row)
                    .map(|r| r.get_chars_at(offset.col))
                    .flatten()
                else {
                    text_buffer.push('\n');
                    continue;
                };

                for c in chars {
                    text_buffer.push(c);
                }
            }

            frame.set_cursor_position((
                view.cursor.col as u16 - offset.col as u16,
                view.cursor.row as u16 - offset.row as u16,
            ));
            frame.render_widget(Paragraph::new(text_buffer), frame.area());
        })?;

        match event::read()? {
            Event::Key(key) => match (key.kind, key.code) {
                (KeyEventKind::Press, KeyCode::Char(c)) => {
                    view.insert_char(c).unwrap();
                }
                (KeyEventKind::Press, KeyCode::Enter) => {
                    view.insert_char('\n').unwrap();
                }
                (KeyEventKind::Press, KeyCode::Tab) => {
                    view.insert("    ").unwrap();
                }
                (KeyEventKind::Press, KeyCode::Backspace) => {
                    view.remove().unwrap();
                }
                (KeyEventKind::Press, KeyCode::Delete) => {
                    view.remove_front().unwrap();
                }

                (KeyEventKind::Press, KeyCode::Esc) => break,

                (KeyEventKind::Press, KeyCode::Up) => {
                    view.move_cursor(MoveCursor::Up(1));
                }
                (KeyEventKind::Press, KeyCode::Down) => {
                    view.move_cursor(MoveCursor::Down(1));
                }
                (KeyEventKind::Press, KeyCode::Left) => {
                    view.move_cursor(MoveCursor::Left(1));
                }
                (KeyEventKind::Press, KeyCode::Right) => {
                    view.move_cursor(MoveCursor::Right(1));
                }

                (KeyEventKind::Press, KeyCode::Home) => {
                    view.cursor.set_column(0);
                }
                (KeyEventKind::Press, KeyCode::End) => {
                    view.cursor.set_column(view.line_end_index(view.cursor.row));
                }

                _ => (),
            },
            Event::Paste(text) => {
                view.insert(&text).unwrap();
            }
            _ => (),
        }
    }

    ratatui::restore();
    Ok(())
}
