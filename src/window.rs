use tui::{
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::buffer::Buffer;

// https://docs.rs/ropey/1.2.0/ropey/struct.Rope.html?search=#method.byte_to_line
//

/// A window/visible buffer
pub struct Window<'a> {
    buffer: &'a Buffer,
    // TODO: need to have some sort of Point object that defines both the
    // of the screen as well as the current location of the cursor in the
    // buffer
    // cursor: Point,
}

fn write_token<'a>(text: &'a str, token: &'static str) -> Span<'a> {
    Span::styled(
        text,
        Style::default().fg(match token {
            "function" => Color::Rgb(246, 199, 255),
            "identifier" => Color::Cyan,
            "string" => Color::Yellow,
            _ => Color::White,
        }),
    )
}

impl<'a> Window<'a> {
    pub fn new(buffer: &'a Buffer) -> Window<'a> {
        Window { buffer: buffer }
    }

    pub fn highlight(&self) -> Vec<Span> {
        let cursor = &mut self.buffer.tree.walk();
        let mut vector: Vec<Span> = vec![];
        let mut token_end = 0;
        loop {
            if cursor.node().kind() == "string" || !cursor.goto_first_child() {
                let start_byte = cursor.node().start_byte();
                if start_byte - token_end != 0 {
                    vector.push(Span::raw(
                        self.buffer
                            .content
                            .slice(token_end..start_byte)
                            .as_str()
                            .unwrap(),
                    ));
                }
                vector.push(write_token(
                    self.buffer
                        .content
                        .slice(start_byte..cursor.node().end_byte())
                        .as_str()
                        .unwrap(),
                    cursor.node().kind(),
                ));
                token_end = cursor.node().end_byte();
                while !cursor.goto_next_sibling() {
                    if !cursor.goto_parent() {
                        return vector;
                    }
                }
            }
        }
    }

    // fn render_with_viewport(&self, x: u32, y: u32, w: u16, h: u16) -> Spans {
    //     let start_byte = self.buffer.content.byte_to_line(x);

    //     // Spans::from(vec![for i in y..y + h {
    //     //     Span::raw("owo")
    //     // }])
    // }

    pub fn get_widget(&self, viewport: Rect) -> Paragraph {
        // let text = Span::raw(self.buffer.content.clone());
        // let text = self.render_with_viewport(0, 0, viewport.width, viewport.height);

        let text = Spans::from(self.highlight());
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(self.buffer.name.clone())
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .wrap(Wrap { trim: true })
    }
}
