use rand::Rng;
use tcod::{colors::*, Console, BackgroundFlag};
use std::cmp;

use crate::{Game, map::Map, PLAYER, Tcod};

/// This is a generic object: the player, a monster, an item, the stairs...
/// It's always represented by a character on screen.
#[derive(Debug, Clone)]
pub struct Object {
   pub x: i32,
   pub  y: i32,
   pub char: char,
   pub  color: Color,
   pub name: String,  
   pub blocks: bool,  
   pub alive: bool,  
   pub fighter: Option<Fighter>,  
   pub ai: Option<Ai>,  
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
            fighter: None,  
            ai: None,  
        }
    }

    
    /// move or attack by the given destination
    pub fn player_move_or_attack(&mut self, dx: i32, dy: i32, game: &Game, objects: &mut [Object]) {
        let x = self.x + dx;
        let y = self.y + dy;
// try to find an attackable object there
let target_id = objects
    .iter()
    .position(|object| object.fighter.is_some() && object.pos() == (x, y));

        if let Some(target_id) = target_id {
            let (player, target) = mut_two(PLAYER, target_id, objects);
            player.attack(target);
        } else {
            move_by(PLAYER, dx, dy, &game.map, objects);
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


    /// return the distance to another object
    pub fn distance_to(&self, other: &Object) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }

    pub fn take_damage(&mut self, damage: i32) {
        // apply damage if possible
        if let Some(fighter) = self.fighter.as_mut() {
            if damage > 0 {
                fighter.hp -= damage;
            }
        }

        // check for death, call the death function
        if let Some(fighter) = self.fighter {
            if fighter.hp <= 0 {
                self.alive = false;
                fighter.on_death.callback(self);
            }
        }
    }

    pub fn attack(&mut self, target: &mut Object) {
        // a simple(ish) formula for attack damage
        let damage = self.fighter.map_or(0, |f| f.power) - target.fighter.map_or(0, |f| f.defense) / 4;
        if damage > 0 {
            // make the target take some damage
            println!(
                "{} attacks {} for {} hit points.",
                self.name, target.name, damage
            );
            target.take_damage(damage);
        } else {
            println!(
                "{} attacks {} but it has no effect!",
                self.name, target.name
            );
        }
    }
}

/// move by the given amount, if the destination is not blocked
pub fn move_by(id: usize, dx: i32, dy: i32, map: &Map, objects: &mut [Object]) {
    let (x, y) = objects[id].pos();
    if !is_blocked(x + dx, y + dy, map, objects) {
        objects[id].set_pos(x + dx, y + dy);
    }
}

pub fn ai_take_turn(monster_id: usize, tcod: &Tcod, game: &Game, objects: &mut [Object]) {
    // a basic monster takes its turn. If you can see it, it can see you
    let (monster_x, monster_y) = objects[monster_id].pos();
    if tcod.fov.is_in_fov(monster_x, monster_y) {
        if objects[monster_id].distance_to(&objects[PLAYER]) >= 2.0 {
            // move towards player if far away
            let (player_x, player_y) = objects[PLAYER].pos();
            move_towards(monster_id, player_x, player_y, &game.map, objects);
        } else if objects[PLAYER].fighter.map_or(false, |f| f.hp > 0) {
            // close enough, attack! (if the player is still alive.)
            let player_id = PLAYER;
            {
                // close enough, attack! (if the player is still alive.)
                let (monster, player) = mut_two(monster_id, PLAYER, objects);
                monster.attack(player);
            }
        } else {
            // just move in a random direction
            move_by(
                monster_id,
                rand::thread_rng().gen_range(-1..1),
                rand::thread_rng().gen_range(-1..1),
                &game.map,
                objects,
            );
        }
    }
}

/// move towards the target
pub fn move_towards(id: usize, target_x: i32, target_y: i32, map: &Map, objects: &mut [Object]) {
    // vector from this object to the target, and distance
    let dx = target_x - objects[id].x;
    let dy = target_y - objects[id].y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    // normalize it to length 1 (preserving direction), then round it and
    // convert to integer so the movement is restricted to the map grid
    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    move_by(id, dx, dy, map, objects);
}



// combat-related properties and methods (monster, player, NPC).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fighter {
    /// maximum hit points (defines the monster's maximum possible health)
    pub max_hp: i32,
    /// current hit points (defines how much hurt the monster is, if its health is 0, the monster dies)
    pub hp: i32,
    /// defense points (defines how much damage the monster can absorb from a physical hite)
    pub defense: i32,
    /// power points (defines how much pysical damage the monster can do)
    pub power: i32,
    /// magic points (defines how much magic damage the monster can do)
    pub magic: i32,
    /// magic defense points (defines how much magic damage the monster can absorb from a magical hit)
    pub magic_defense: i32,
    /// on_death function (called when the monster dies)
    pub on_death: DeathCallback,  
}

/// Mutably borrow two *separate* elements from the given slice.
/// Panics when the indexes are equal or out of bounds.
fn mut_two<T>(first_index: usize, second_index: usize, items: &mut [T]) -> (&mut T, &mut T) {
    assert!(first_index != second_index);
    let split_at_index = cmp::max(first_index, second_index);
    let (first_slice, second_slice) = items.split_at_mut(split_at_index);
    if first_index < second_index {
        (&mut first_slice[first_index], &mut second_slice[0])
    } else {
        (&mut second_slice[0], &mut first_slice[second_index])
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Ai {
    Basic,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeathCallback {
    Player,
    Monster,
}

impl DeathCallback {
    fn callback(self, object: &mut Object) {
        use DeathCallback::*;
        let callback: fn(&mut Object) = match self {
            Player => player_death,
            Monster => monster_death,
        };
        callback(object);
    }
}

fn player_death(player: &mut Object) {
    // the game ended!
    println!("Your fragle body smashes into blood and guts! your unkindred soul will be in torment... forever. lil' warm unkindred heart...");
    
    player.alive = false;

    // for added effect, transform the player into a corpse!
    player.char = '%';
    player.color = DARK_RED;
}

fn monster_death(monster: &mut Object) {
    // transform it into a nasty corpse! it doesn't block, can't be
    // attacked and doesn't move
    println!("{} dies!", monster.name);
    monster.char = '%';
    monster.color = DARK_RED;
    monster.blocks = false;
    monster.fighter = None;
    monster.ai = None;
    monster.name = format!("Corpse of {}", monster.name);
}