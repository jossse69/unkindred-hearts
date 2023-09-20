use tcod::{colors::*, Console, BackgroundFlag};

use crate::{Game, map::Map, PLAYER};

/// This is a generic object: the player, a monster, an item, the stairs...
/// It's always represented by a character on screen.
#[derive(Debug, Clone)]
pub(crate) struct Object {
   pub x: i32,
   pub  y: i32,
   pub char: char,
   pub  color: Color,
   pub name: String,  
   pub blocks: bool,  
   pub alive: bool,  
}

pub(crate) fn is_blocked(x: i32, y: i32, map: &Map, objects: &[Object]) -> bool {
    // first test the map tile
    if map[x as usize][y as usize].blocked {
        return true;
    }
    // now check for any blocking objects
    objects
        .iter()
        .any(|object| object.blocks && object.pos() == (x, y))
}

impl Object {
    /// create a new object
    pub fn new(x: i32, y: i32, char: char, name: &str, color: Color, blocks: bool) -> Self {
        Object {
            x: x,
            y: y,
            char: char,
            color: color,
            name: name.into(),
            blocks: blocks,
            alive: false,
        }
    }

    /// move by the given amount, if the destination is not blocked
    pub fn move_by(&mut self,id: usize, dx: i32, dy: i32, map: &Map, objects: &mut [Object]) {
        let (x, y) = objects[id].pos();
        if !is_blocked(x + dx, y + dy, map, objects) {
            objects[id].set_pos(x + dx, y + dy);
        }
    }
    /// move or attack by the given destination
    pub fn player_move_or_attack(&mut self, dx: i32, dy: i32, game: &Game, objects: &mut [Object]) {
        // the coordinates the player is moving to/attacking
        let x = self.x + dx;
        let y = self.y + dy;
    
        // try to find an attackable object there
        let target_id = Game::find_object(x, y, objects);

    
        // attack if target found, move otherwise
        if target_id.is_some(){
            println!("{} attacks {}!", self.name, objects[target_id.unwrap()].name);
        } else {
            self.move_by(PLAYER, dx, dy, &game.map, objects);
        }
    }

    /// set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    /// set the position of this object
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// return the position
    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    
}

