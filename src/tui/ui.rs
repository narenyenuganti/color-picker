use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::tui::app::App;

pub fn render(f: &mut Frame, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_chunks[0]);

    render_saved_colors(f, app, columns[0]);
    render_color_stream(f, app, columns[1]);
    render_bottom_bar(f, app, main_chunks[1]);
}

fn render_saved_colors(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let title = format!(" Saved Colors ({}) ", app.store.saved_colors.len());

    let items: Vec<ListItem> = app
        .store
        .saved_colors
        .iter()
        .map(|sc| {
            let color = sc.to_color();
            let rcolor = color.to_ratatui_color();
            ListItem::new(Line::from(vec![
                Span::styled("● ", Style::default().fg(rcolor)),
                Span::raw(&sc.hex),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    state.select(app.selected_index);
    f.render_stateful_widget(list, area, &mut state);
}

fn render_color_stream(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .color_stream
        .iter()
        .map(|c| {
            let rcolor = c.to_ratatui_color();
            ListItem::new(Line::from(vec![
                Span::styled("● ", Style::default().fg(rcolor)),
                Span::styled(c.to_hex(), Style::default().fg(rcolor)),
                Span::raw(" "),
                Span::styled("████", Style::default().fg(rcolor)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Color Stream "),
    );

    f.render_widget(list, area);
}

fn render_bottom_bar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let current_text = match &app.current_color {
        Some(c) => {
            let rcolor = c.to_ratatui_color();
            Line::from(vec![
                Span::raw(" Current: "),
                Span::styled("● ", Style::default().fg(rcolor)),
                Span::styled(
                    c.to_hex(),
                    Style::default().fg(rcolor).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                status_span(app),
                Span::raw("  "),
                Span::styled(
                    "s:save  d:del  c:clear  y:copy  q:quit",
                    Style::default().fg(ratatui::style::Color::DarkGray),
                ),
            ])
        }
        None => Line::from(vec![
            Span::raw(" Waiting for color..."),
            Span::raw("  "),
            Span::styled(
                "q:quit",
                Style::default().fg(ratatui::style::Color::DarkGray),
            ),
        ]),
    };

    let bar = Paragraph::new(current_text)
        .block(Block::default().borders(Borders::ALL).title(" Status "));

    f.render_widget(bar, area);
}

fn status_span(app: &App) -> Span<'_> {
    match &app.status_message {
        Some(msg) if app.clear_confirm => {
            Span::styled(msg.as_str(), Style::default().fg(ratatui::style::Color::Red))
        }
        Some(msg) => Span::styled(
            msg.as_str(),
            Style::default().fg(ratatui::style::Color::Yellow),
        ),
        None => Span::raw(""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use tempfile::TempDir;

    #[test]
    fn test_render_empty_state_does_not_panic() {
        let dir = TempDir::new().unwrap();
        let app = App::new(dir.path().join("colors.json"));
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }

    #[test]
    fn test_render_with_colors_does_not_panic() {
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().join("colors.json"));
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();
        app.push_color(Color::new(0, 255, 0));
        app.selected_index = Some(0);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }

    #[test]
    fn test_render_with_status_message() {
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().join("colors.json"));
        app.push_color(Color::new(255, 0, 0));
        app.set_status("Copied!");

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }

    #[test]
    fn test_render_with_clear_confirm() {
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().join("colors.json"));
        app.clear_confirm = true;
        app.set_status("Press c again to clear all");

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }

    #[test]
    fn test_render_with_many_stream_colors() {
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().join("colors.json"));
        for i in 0..50 {
            app.push_color(Color::new(i as u8, i as u8 * 2, 255 - i as u8));
        }

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }

    #[test]
    fn test_render_narrow_terminal() {
        let dir = TempDir::new().unwrap();
        let mut app = App::new(dir.path().join("colors.json"));
        app.push_color(Color::new(255, 0, 0));
        app.save_current_color();

        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
    }
}
