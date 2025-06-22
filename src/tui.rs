use ratatui::{Frame, layout::Rect};
use crossterm::event::Event;

pub trait WindowLayout {
    // render the layout to the terminal
    fn draw(&mut self, f: &mut Frame, area: Rect);

    // handle events like key presses or mouse clicks
    fn handle_event(&mut self, event: Event);

    // get the current area of the layout
    fn get_size(&self) -> (u16, u16);

    // set the size of the layout
    fn set_size(&mut self, size: (u16, u16));
}