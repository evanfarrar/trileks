#![feature(old_path)]

extern crate piston;
extern crate ai_behavior;
extern crate sprite;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate rand;
extern crate uuid;


use std::cell::RefCell;
use std::rc::Rc;
use piston::input::Button;
use piston::input::keyboard::Key;

use rand::Rng;
use rand::distributions::{IndependentSample, Range};


use uuid::Uuid;

use sprite::*;
use graphics::ImageSize;
use ai_behavior::{
    Action,
    Sequence,
};

use sdl2_window::Sdl2Window;
use opengl_graphics::{
    GlGraphics,
    OpenGL,
    Texture,
};

fn random_grid_position<R: Rng>(random_horizontal: &Range<usize>, random_vertical: &Range<usize>, rng: &mut R) -> (usize, usize) {
    (random_horizontal.ind_sample(rng), random_vertical.ind_sample(rng))
}

fn grid_position_as_scene_position(grid_position: (usize,usize)) -> (f64,f64) {
    let (grid_horizontal,grid_vertical) = grid_position;
    ((grid_horizontal*100+50) as f64, (grid_vertical*100+50) as f64)
}

fn scene_position_as_grid_position(scene_position: (f64,f64)) -> (usize,usize) {
    let (scene_horizontal,scene_vertical) = scene_position;
    ((((scene_horizontal as usize)-50)/100), (((scene_vertical as usize)-50)/100))
}

fn collides<I: ImageSize>(sprite1: &Sprite<I>, sprite2: &Sprite<I>) -> bool {
    let sprite1_position = scene_position_as_grid_position(sprite1.position());
    let sprite2_position = scene_position_as_grid_position(sprite2.position());
    sprite1_position == sprite2_position
}

struct ActorInProgress {
    grid_x: usize,
    grid_y: usize,
    alive: bool,
}
impl ActorInProgress {
    fn scene_x(&self) -> f64 {
        (self.grid_x*100+50) as f64
    }
    fn scene_y(&self) -> f64 {
        (self.grid_y*100+50) as f64
    }
    fn to_actor(&self, sprite_uuid: uuid::Uuid) -> Actor {
        Actor{ grid_x: self.grid_x, grid_y: self.grid_y, sprite_uuid: sprite_uuid, alive: self.alive }
    }
}
struct Actor {
    grid_x: usize,
    grid_y: usize,
    sprite_uuid: uuid::Uuid,
    alive: bool,
}
impl Actor {
    fn collides(&self, other_actor: &Actor) -> bool {
        other_actor.grid_x == self.grid_x && other_actor.grid_y == self.grid_y
    }

    fn scene_position(&self) -> (f64,f64) {
        (self.scene_x(), self.scene_y())
    }

    fn scene_x(&self) -> f64 {
        (self.grid_x*100+50) as f64
    }

    fn scene_y(&self) -> f64 {
        (self.grid_y*100+50) as f64
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let random_horizontal = Range::new(0, 8);
    let random_vertical = Range::new(0, 6);
    let (width, height) = (800, 600);
    let opengl = OpenGL::_3_2;
    let window = Sdl2Window::new(
        opengl,
        piston::window::WindowSettings {
            title: "Trileks".to_string(),
            size: [width, height],
            fullscreen: false,
            exit_on_esc: true,
            samples: 0,
        }
    );

    let mut scene = Scene::new();

    let tex = Path::new("./frog.png");
    let tex = Rc::new(Texture::from_path(&tex).unwrap());
    let mut frog_texture = Sprite::from_texture(tex.clone());

    let (pos_h, pos_v) = random_grid_position(&random_horizontal, &random_vertical, &mut rng);
    let frog_in_progress = ActorInProgress{ grid_x: pos_h, grid_y: pos_v, alive: true  };
    frog_texture.set_position(frog_in_progress.scene_x(), frog_in_progress.scene_y());
    let frog = frog_in_progress.to_actor(scene.add_child(frog_texture));

    let tex = Path::new("./robot.png");
    let tex = Rc::new(Texture::from_path(&tex).unwrap());
    let mut robot_texture = Sprite::from_texture(tex.clone());

    let (pos_h, pos_v) = random_grid_position(&random_horizontal, &random_vertical, &mut rng);
    let robot_in_progress = ActorInProgress{ grid_x: pos_h, grid_y: pos_v, alive: true  };
    robot_texture.set_position(robot_in_progress.scene_x(), robot_in_progress.scene_y());
    let robot = robot_in_progress.to_actor(scene.add_child(robot_texture));

    let ref mut gl = GlGraphics::new(opengl);
    let window = RefCell::new(window);
    let mut frog_alive = true;

    for e in piston::events(&window) {
        use piston::event::{ PressEvent, RenderEvent };
        scene.event(&e);
        if let Some(args) = e.render_args() {
            use graphics::*;
            gl.draw([0, 0, args.width as i32, args.height as i32], |c, gl| {
                graphics::clear([0.7, 0.5, 0.8, 1.0], gl);
                scene.draw(&c, gl);
            });
        }
        let collision = collides(scene.child(frog.sprite_uuid.clone()).unwrap(),scene.child(robot.sprite_uuid.clone()).unwrap());
        if collision && frog_alive {
            println!("COLLISION");
            frog_alive = false;
            let seq = Sequence(vec![
                Action(Blink(1.0, 5)),
                Action(ScaleBy(0.5, 0.0, -0.5)),
                Action(FadeOut(1.0)),
            ]);
            scene.run(frog.sprite_uuid.clone(), &seq);
        }
        if frog_alive {
            if let Some(Button::Keyboard(key)) = e.press_args() {
                let mut frog_moved = false;
                if key == Key::Q {
                    scene.run(frog.sprite_uuid.clone(),&Action(MoveBy(0.5, -100.0, -100.0)));
                    frog_moved = true;
                }
                if key == Key::W {
                    scene.run(frog.sprite_uuid.clone(),&Action(MoveBy(0.5, 0.0, -100.0)));
                    frog_moved = true;
                }
                if key == Key::E {
                    scene.run(frog.sprite_uuid.clone(),&Action(MoveBy(0.5, 100.0, -100.0)));
                    frog_moved = true;
                }
                if key == Key::A {
                    scene.run(frog.sprite_uuid.clone(),&Action(MoveBy(0.5, -100.0, 0.0)));
                    frog_moved = true;
                }
                if key == Key::D {
                    scene.run(frog.sprite_uuid.clone(),&Action(MoveBy(0.5, 100.0, 0.0)));
                    frog_moved = true;
                }
                if key == Key::Z {
                    scene.run(frog.sprite_uuid.clone(),&Action(MoveBy(0.5, -100.0, 100.0)));
                    frog_moved = true;
                }
                if key == Key::X {
                    scene.run(frog.sprite_uuid.clone(),&Action(MoveBy(0.5, 0.0, 100.0)));
                    frog_moved = true;
                }
                if key == Key::C {
                    scene.run(frog.sprite_uuid.clone(),&Action(MoveBy(0.5, 100.0, 100.0)));
                    frog_moved = true;
                }
                if key == Key::T {
                    let (pos_h, pos_v) = grid_position_as_scene_position(random_grid_position(&random_horizontal, &random_vertical, &mut rng));
                    scene.run(frog.sprite_uuid.clone(),&Action(MoveTo(0.0, pos_h, pos_v)));
                }
                if frog_moved {
                    let (frog_pos_h, frog_pos_v) = scene.child(frog.sprite_uuid.clone()).unwrap().position();
                    let (robot_pos_h, robot_pos_v) = scene.child(robot.sprite_uuid.clone()).unwrap().position();
                    let mut move_h = 0.0;
                    let mut move_v = 0.0;
                    if frog_pos_h > robot_pos_h {
                        move_h = 100.0;
                    } else if frog_pos_h < robot_pos_h {
                        move_h = -100.0;
                    }
                    if frog_pos_v > robot_pos_v {
                        move_v = 100.0;
                    } else if frog_pos_v < robot_pos_v {
                        move_v = -100.0;
                    }
                    scene.run(robot.sprite_uuid.clone(), &Action(MoveBy(0.75, move_h, move_v)));
                }
            }
        }
    }
}
