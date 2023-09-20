use object::DeathCallback;
use object::Fighter;
use object::ai_take_turn;
use tcod::colors::*;
use tcod::console::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;

mod object;
use object::Object;

mod map;
use map::{Map, make_map, render_map};
use tcod::map::{FovAlgorithm, Map as FovMap};

mod ui;

use ui::{render_bar, Messages};

// player will always be the first object
const PLAYER: usize = 0;
// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const LIMIT_FPS: i32 = 30; 
// size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 43; 

// sizes and coordinates relevant for the GUI
const BAR_WIDTH: i32 = 20;
const PANEL_HEIGHT: i32 = 7;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;


const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic; // default FOV algorithm
const FOV_LIGHT_WALLS: bool = true; // light walls or not
const FOV_RADIUS: i32 = 8;

const MSG_X: i32 = BAR_WIDTH + 2;
const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;

pub struct Tcod {
    root: Root,
    con: Offscreen,
    panel: Offscreen,
    fov: FovMap,    
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}
pub struct Game {
    map: Map,
    messages: Messages,
}

impl Game {
    pub fn find_object(x: i32, y: i32, objects: &[Object]) -> Option<usize> {
        // iterate through the objects array and return the first index that matches
        objects.iter().position(|object| object.pos() == (x, y))
    }
}

fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], map: &mut Map, fov_recompute: bool) {
    // render the map
    render_map(tcod, map, game);

    if fov_recompute {
        // recompute FOV if needed (the player moved or something)
        let player = &objects[PLAYER];
        tcod.fov
            .compute_fov(player.x, player.y, FOV_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    

    let mut to_draw: Vec<_> = objects
    .iter()
    .filter(|o| tcod.fov.is_in_fov(o.x, o.y))
    .collect();
    // sort so that non-blocking objects come first
    to_draw.sort_by(|o1, o2| o1.blocks.cmp(&o2.blocks));
    // draw the objects in the list
    for object in &to_draw {
        object.draw(&mut tcod.con);
    }


    
    // blit the contents of "con" to the root console
    blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );

    // prepare to render the GUI panel
    tcod.panel.set_default_background(BLACK);
    tcod.panel.clear();

    // show the player's stats
    let hp = objects[PLAYER].fighter.map_or(0, |f| f.hp);
    let max_hp = objects[PLAYER].fighter.map_or(0, |f| f.max_hp);
    render_bar(
        &mut tcod.panel,
        1,
        1,
        BAR_WIDTH,
        "HP",
        hp,
        max_hp,
        LIGHT_RED,
        DARKER_RED,
    );

    // print the game messages, one line at a time
    let mut y = MSG_HEIGHT as i32;
    for &(ref msg, color) in game.messages.iter().rev() {
        let msg_height = tcod.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        y -= msg_height;
        if y < 0 {
            break;
        }
        tcod.panel.set_default_foreground(color);
        tcod.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    }

    // blit the contents of `panel` to the root console
    blit(
        &tcod.panel,
        (0, 0),
        (SCREEN_WIDTH, PANEL_HEIGHT),
        &mut tcod.root,
        (0, PANEL_Y),
        1.0,
        1.0,
    );
}

fn handle_keys(tcod: &mut Tcod, player: &mut Object, mut game: &mut Game, objects: &mut Vec<Object>) -> PlayerAction {
    use PlayerAction::*;
    let key = tcod.root.wait_for_keypress(true);
    let player_alive = objects[PLAYER].alive;
    match (key, key.text(), player_alive) {
        (Key { code: Enter, alt: true, .. }, _, _) => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
            DidntTakeTurn
        }
        (Key { code: Escape, .. }, _, _) => Exit, // exit game

        // movement keys
        (Key { code: NumPad8, .. }, _, true) => {
            player.player_move_or_attack(0, -1, &mut game, objects);
            TookTurn
        }
        (Key { code: NumPad2, .. }, _, true) => {
            player.player_move_or_attack(0, 1, &mut game, objects);
            TookTurn
        }
        (Key { code: NumPad4, .. }, _, true) => {
            player.player_move_or_attack(-1, 0, &mut game, objects);
            TookTurn
        }
        (Key { code: NumPad6, .. }, _, true) => {
            player.player_move_or_attack(1, 0, &mut game, objects);
            TookTurn
        }
        (Key { code: NumPad7, .. }, _, true) => {
            player.player_move_or_attack(-1, -1, &mut game, objects);
            TookTurn
        }
        (Key { code: NumPad9, .. }, _, true) => {
            player.player_move_or_attack(1, -1, &mut game, objects);
            TookTurn
        }
        (Key { code: NumPad1, .. }, _, true) => {
            player.player_move_or_attack(-1, 1, &mut game, objects);
            TookTurn
        }
        (Key { code: NumPad3, .. }, _, true) => {
            player.player_move_or_attack(1, 1, &mut game, objects);
            TookTurn
        }

        _ => DidntTakeTurn
    }
}
fn main() {

    // create object representing the player
    let mut player = Object::new(25, 23, '@', "player", YELLOW, true);
    player.alive = true;
    player.fighter = Some(Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        power: 5,
        magic_defense: 0,
        magic: 0,
        on_death: DeathCallback::Player

    });
    // force FOV "recompute" first time through the game loop
    let mut previous_player_position = (-1, -1);

     // the list of objects
    let mut objects = vec![player];

    let mut map = make_map(MAP_WIDTH, MAP_HEIGHT, &mut objects);

    let mut game = Game {
        map: map.clone(),
        messages: Messages::new()
    };

    



    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);



    let root = Root::initializer()
    .font("assets/terminal8x8_gs_ro.png", FontLayout::AsciiInRow)
    .font_type(FontType::Greyscale)
    .size(SCREEN_WIDTH, SCREEN_HEIGHT)
    .title("unkindred hearts")
    .init();

    let mut tcod = Tcod { root, con, fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT) , panel: Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT) };

    

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize].blocked,
            );
        }
    }

    let mut player_action = PlayerAction::DidntTakeTurn; // Declare and initialize player_action before the loop

    // a warm welcoming message!
    game.messages.add("Welcome, unkindred soul. the void is trying to source of the Fostering Benevolence!", YELLOW);
    game.messages.add("this will spread the void to the rest of the world! save us all before it!", YELLOW);

    render_all(&mut tcod, &mut game, &objects, &mut map, true);
    tcod::system::set_fps(LIMIT_FPS);
    while !tcod.root.window_closed() {
        tcod.con.set_default_foreground(WHITE);
        tcod.con.clear();

        let fov_recompute = previous_player_position != (objects[PLAYER].pos());
        render_all(&mut tcod, &mut game, &objects, &mut map, fov_recompute);

        tcod.root.flush();
        tcod.root.wait_for_keypress(true);
        // handle keys and exit game if needed
        previous_player_position = objects[PLAYER].pos();
        {
            let mut player_clone = objects[PLAYER].clone();
            player_action = handle_keys(&mut tcod, &mut player_clone, &mut game, &mut objects);
            if player_action == PlayerAction::Exit {
                break;
            }
        }

        // let monsters take their turn
        if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
            for id in 0..objects.len() {
                // only if object is not player
                if id != PLAYER {
                    let object = &mut objects[id];
                    if object.ai.is_some() {
                        ai_take_turn(id, &tcod, &mut game, &mut objects);
                    }
                }
            }
        }
    }
}
