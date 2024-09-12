use core::{
    action::Action,
    cursor::{Cursor, MoveCursor},
    view::{TextView, View as _},
    ViewMode,
};

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Color, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();

    let mut view = TextView::from_str("");
    let mut mode = ViewMode::Normal;
    let mut action: Option<Action> = None;

    let mut offset = Cursor::default();

    loop {
        terminal.draw(|frame| {
            let [text_area, status_area, action_area] = Layout::vertical([
                Constraint::Fill(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(frame.area());

            let [marker_area, linenum_area, _spacer, text_area] = Layout::horizontal([
                Constraint::Length(1),
                Constraint::Length(4),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(text_area);

            if offset.col > 0 && view.cursor().col < offset.col + 5 {
                offset.col = view.cursor().col - 5;
            } else if view.cursor().col - offset.col > text_area.width as usize - 5 {
                offset.col = view.cursor().col + 5 - text_area.width as usize;
            }

            if offset.row > 0 && view.cursor().row < offset.row + 2 {
                offset.row = view.cursor().row - 2;
            } else if view.cursor().row - offset.row > text_area.height as usize - 2 {
                offset.row = view.cursor().row + 2 - text_area.height as usize;
            }

            let mut text_buffer = Text::default();
            let mut linenum_buffer = Text::default();
            for row in
                offset.row..(offset.row + text_area.height as usize).min(view.buffer.len_lines())
            {
                let linenum = if row == view.cursor().row {
                    Line::styled(format!("{row}\n"), Color::White)
                } else if matches!(mode, ViewMode::Insert) {
                    Line::styled(format!("{row}\n"), Color::DarkGray)
                } else {
                    let distance = (row as isize - view.cursor().row as isize).abs();
                    Line::styled(format!("{distance}\n"), Color::DarkGray)
                };
                linenum_buffer.push_line(linenum);

                let Some(chars) = view.buffer.get_line(row).map(|r| r.chars()) else {
                    text_buffer.push_line(Line::default());
                    continue;
                };

                let content = chars.collect::<String>();
                text_buffer.push_line(Line::raw(content));
            }

            frame.set_cursor_position((
                view.cursor().col as u16 - offset.col as u16 + text_area.x,
                view.cursor().row as u16 - offset.row as u16 + text_area.y,
            ));
            frame.render_widget(
                Paragraph::new(linenum_buffer).dark_gray().right_aligned(),
                linenum_area,
            );
            frame.render_widget(
                Paragraph::new(text_buffer).scroll((0, offset.col as u16)),
                text_area,
            );

            // Status
            frame.render_widget(
                Paragraph::new(format!("{} {}", view.cursor(), mode))
                    .white()
                    .on_blue(),
                status_area,
            );

            // Action
            if let Some(action) = action.as_ref() {
                frame.render_widget(Paragraph::new(format!("{action}")), action_area);
            }
        })?;

        let event = event::read()?;
        match &mut action {
            Some(mut_action) => match event {
                Event::Key(key) => match (key.kind, key.code) {
                    (KeyEventKind::Press, KeyCode::Char(c)) => {
                        mut_action.push_char(c);
                    }
                    (KeyEventKind::Press, KeyCode::Backspace) => {
                        mut_action.remove_char();
                    }
                    (KeyEventKind::Press, KeyCode::Enter) => {
                        mut_action.execute();
                    }
                    (KeyEventKind::Press, KeyCode::Esc) => {
                        action = None;
                    }
                    _ => (),
                },
                _ => (),
            },
            None => {
                match event {
                    Event::Key(key) => match (&mode, key.kind, key.code) {
                        // ---- Normal ----
                        (ViewMode::Normal, KeyEventKind::Press, KeyCode::Char('d')) => {
                            view.remove(false).unwrap();
                        }
                        (ViewMode::Normal, KeyEventKind::Press, KeyCode::Char('c')) => {
                            view.remove(false).unwrap();
                            mode = ViewMode::Insert;
                            ratatui::crossterm::execute!(
                                std::io::stdout(),
                                ratatui::crossterm::cursor::SetCursorStyle::SteadyBar
                            )
                            .unwrap();
                        }
                        (ViewMode::Normal, KeyEventKind::Press, KeyCode::Char('o')) => {
                            view.move_cursor(MoveCursor::End);
                            view.insert_char('\n').unwrap();

                            mode = ViewMode::Insert;
                            ratatui::crossterm::execute!(
                                std::io::stdout(),
                                ratatui::crossterm::cursor::SetCursorStyle::SteadyBar
                            )
                            .unwrap();
                        }

                        (ViewMode::Normal, KeyEventKind::Press, KeyCode::Char('i')) => {
                            mode = ViewMode::Insert;
                            ratatui::crossterm::execute!(
                                std::io::stdout(),
                                ratatui::crossterm::cursor::SetCursorStyle::SteadyBar
                            )
                            .unwrap();
                        }
                        (ViewMode::Normal, KeyEventKind::Press, KeyCode::Char('v')) => {
                            mode = ViewMode::Select;
                            ratatui::crossterm::execute!(
                                std::io::stdout(),
                                ratatui::crossterm::cursor::SetCursorStyle::SteadyUnderScore
                            )
                            .unwrap();
                        }

                        (ViewMode::Normal, KeyEventKind::Press, KeyCode::Esc) => {
                            break;
                        }

                        // ---- Insert ----
                        (ViewMode::Insert, KeyEventKind::Press, KeyCode::Char(c)) => {
                            view.insert_char(c).unwrap();
                        }
                        (ViewMode::Insert, KeyEventKind::Press, KeyCode::Enter) => {
                            view.insert_char('\n').unwrap();
                        }
                        (ViewMode::Insert, KeyEventKind::Press, KeyCode::Tab) => {
                            view.insert("    ").unwrap();
                        }
                        (ViewMode::Insert, KeyEventKind::Press, KeyCode::Backspace) => {
                            view.remove(true).unwrap();
                        }
                        (ViewMode::Insert, KeyEventKind::Press, KeyCode::Delete) => {
                            view.remove(false).unwrap();
                        }

                        // ---- Dual ----
                        (
                            ViewMode::Insert | ViewMode::Select,
                            KeyEventKind::Press,
                            KeyCode::Esc,
                        ) => {
                            mode = ViewMode::Normal;
                            ratatui::crossterm::execute!(
                                std::io::stdout(),
                                ratatui::crossterm::cursor::SetCursorStyle::SteadyBlock
                            )
                            .unwrap();
                        }
                        (
                            ViewMode::Normal | ViewMode::Select,
                            KeyEventKind::Press,
                            KeyCode::Char(':'),
                        ) => {
                            action = Some(Action::Command {
                                text: String::new(),
                            });
                        }

                        // ---- Global ----
                        (_, KeyEventKind::Press, KeyCode::Up)
                        | (
                            ViewMode::Normal | ViewMode::Select,
                            KeyEventKind::Press,
                            KeyCode::Char('k'),
                        ) => {
                            view.move_cursor(MoveCursor::Up(1));
                        }
                        (_, KeyEventKind::Press, KeyCode::Down)
                        | (
                            ViewMode::Normal | ViewMode::Select,
                            KeyEventKind::Press,
                            KeyCode::Char('j'),
                        ) => {
                            view.move_cursor(MoveCursor::Down(1));
                        }
                        (_, KeyEventKind::Press, KeyCode::Left)
                        | (
                            ViewMode::Normal | ViewMode::Select,
                            KeyEventKind::Press,
                            KeyCode::Char('h'),
                        ) => {
                            view.move_cursor(MoveCursor::Left(1));
                        }
                        (_, KeyEventKind::Press, KeyCode::Right)
                        | (
                            ViewMode::Normal | ViewMode::Select,
                            KeyEventKind::Press,
                            KeyCode::Char('l'),
                        ) => {
                            view.move_cursor(MoveCursor::Right(1));
                        }

                        (_, KeyEventKind::Press, KeyCode::Home) => {
                            view.move_cursor(MoveCursor::Home)
                        }
                        (_, KeyEventKind::Press, KeyCode::End) => {
                            view.move_cursor(MoveCursor::End);
                        }

                        _ => (),
                    },
                    Event::Paste(text) => {
                        view.insert(&text).unwrap();
                    }
                    _ => (),
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}
