use tcod::colors::*;
use tcod::console::*;

use crate::Game;
use crate::Tcod;
use crate::object::Object;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};

/// A tile of the map and its properties
#[derive(Clone, Copy, Debug)]
pub(crate) struct Tile {
    pub blocked: bool,
    pub block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
        }
    }
}

pub(crate) type Map = Vec<Vec<Tile>>;

pub(crate) fn make_map(width: i32, height: i32) -> Map {
    // fill map with "unblocked" tiles
    let mut map = vec![vec![Tile::empty(); height as usize]; width as usize];

    // place two pillars to test the map
    map[30][22] = Tile::wall();
    map[50][22] = Tile::wall();

    map
}

pub(crate) fn render_map(tcod: &mut Tcod, map: &Map, game: &Game) {
    // go through all tiles, and set their background color
    for y in 0..map[0].len() {
        for x in 0..map.len() {
            let wall = map[x][y].block_sight; // Corrected indexing
            if wall {
                tcod.con.set_char_foreground(x as i32, y as i32, COLOR_DARK_WALL);
                tcod.con.set_char(x as i32, y as i32, '#');
                    
            } else {
                tcod.con.set_char_foreground(x as i32, y as i32, COLOR_DARK_GROUND);
                tcod.con.set_char(x as i32, y as i32, '.');
            }
        }
    }
}
