use tcod::{colors::*, Console, BackgroundFlag};

use crate::Game;

/// This is a generic object: the player, a monster, an item, the stairs...
/// It's always represented by a character on screen.
pub(crate) struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object { x, y, char, color }
    }

    /// move by the given amount, if the destination is not blocked
    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {  
        if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {  
            self.x += dx;  
            self.y += dy;
        }
    }

    /// set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}