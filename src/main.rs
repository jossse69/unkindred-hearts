use tcod::colors::*;
use tcod::console::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;

mod object;
use object::Object;

mod map;
use map::{Map, make_map, render_map};

// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const LIMIT_FPS: i32 = 20; // 20 frames-per-second maximum

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

struct Tcod {
    root: Root,
    con: Offscreen,
}

struct Game {
    map: Map,
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object], map: &Map) {
    // render the map
    render_map(tcod, map, game);

    // draw all objects in the list
    for object in objects {
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
}

fn handle_keys(tcod: &mut Tcod, player: &mut Object, game: &mut Game) -> bool {
    let key = tcod.root.wait_for_keypress(true);
    match key {
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        }
        Key { code: Escape, .. } => return true, // exit game

        // movement keys
        Key { code: NumPad8, .. } => player.move_by(0, -1, game),
        Key { code: NumPad2, .. } => player.move_by(0, 1, game),
        Key { code: NumPad4, .. } => player.move_by(-1, 0, game),
        Key { code: NumPad6, .. } => player.move_by(1, 0, game),
        Key { code: NumPad7, .. } => player.move_by(-1, -1, game),
        Key { code: NumPad9, .. } => player.move_by(1, -1, game),
        Key { code: NumPad1, .. } => player.move_by(-1, 1, game),
        Key { code: NumPad3, .. } => player.move_by(1, 1, game),
        Key { code: NumPad5, .. } => player.move_by(-1, 1, game),

        _ => {}
    }
    false
}

fn main() {
    let mut game = Game {
        // generate map (at this point it's not drawn to the screen)
        map: make_map(MAP_WIDTH, MAP_HEIGHT),
    };

    let mut player_x = SCREEN_WIDTH / 2;
    let mut player_y = SCREEN_HEIGHT / 2;

    // create object representing the player
    let mut player = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', YELLOW);

    // create an NPC
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', GREEN);

    // the list of objects with those two
    let mut objects = [player, npc];

    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);

    let mut map = make_map(MAP_WIDTH, MAP_HEIGHT);

    let root = Root::initializer()
    .font("assets/terminal8x8_gs_ro.png", FontLayout::AsciiInRow)
    .font_type(FontType::Greyscale)
    .size(SCREEN_WIDTH, SCREEN_HEIGHT)
    .title("unkindred hearts")
    .init();

    let mut tcod = Tcod { root, con };

    tcod::system::set_fps(LIMIT_FPS);
    while !tcod.root.window_closed() {
        tcod.con.set_default_foreground(WHITE);
        tcod.con.clear();
        // render the screen
        render_all(&mut tcod, &game, &objects, &map);
        // blit the contents of "con" to the root console and present it
        blit(
            &tcod.con,
            (0, 0),
            (SCREEN_WIDTH, SCREEN_HEIGHT),
            &mut tcod.root,
            (0, 0),
            1.0,
            1.0,
        );
        tcod.root.flush();
        tcod.root.wait_for_keypress(true);
        // handle keys and exit game if needed
        let player = &mut objects[0];
        let exit = handle_keys(&mut tcod, player, &mut game);
    }
}
