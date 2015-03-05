#![feature(old_path)]
#![feature(std_misc)] 
#![feature(core)]

extern crate piston;
extern crate ai_behavior;
extern crate sprite;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate rand;
extern crate uuid;
extern crate core;

use std::cell::{RefCell, BorrowState};
use std::rc::Rc;
use piston::input::Button;
use piston::input::keyboard::Key;
use core::ops::Deref;
use rand::Rng;
use rand::distributions::{IndependentSample, Range};
use uuid::Uuid;
use sprite::*;
use ai_behavior::{
    Action,
    Sequence,
    Wait,
};
use sdl2_window::Sdl2Window;
use opengl_graphics::{
    GlGraphics,
    OpenGL,
    Texture,
};

struct ActorInProgress {
    grid_x: usize,
    grid_y: usize,
    alive: bool,
    sprite_name: &'static str,
}
impl ActorInProgress {
    fn scene_x(&self) -> f64 {
        (self.grid_x*100+50) as f64
    }
    fn scene_y(&self) -> f64 {
        (self.grid_y*100+50) as f64
    }
    fn to_actor(&self, sprite_uuid: uuid::Uuid) -> Actor {
        Actor{
            grid_x: self.grid_x,
            grid_y: self.grid_y,
            sprite_uuid: sprite_uuid.clone(),
            alive: self.alive,
            moved: false,
        }
    }
}

struct Actor {
    grid_x: usize,
    grid_y: usize,
    sprite_uuid: uuid::Uuid,
    alive: bool,
    moved: bool,
}
impl Actor {
    fn collides(&self, other_actor: &Actor) -> bool {
        other_actor.grid_x == self.grid_x && other_actor.grid_y == self.grid_y
    }

    fn scene_x(&self) -> f64 {
        (self.grid_x*100+50) as f64
    }

    fn scene_y(&self) -> f64 {
        (self.grid_y*100+50) as f64
    }

    fn move_towards(&mut self, other_actor: &Actor) {
        let (frog_pos_h, frog_pos_v) = (other_actor.grid_x,other_actor.grid_y);
        let (robot_pos_h, robot_pos_v) = (self.grid_x,self.grid_y);

        if frog_pos_h > robot_pos_h {
            self.grid_x = self.grid_x + 1;
        } else if frog_pos_h < robot_pos_h {
            self.grid_x = self.grid_x - 1;
        }
        if frog_pos_v > robot_pos_v {
            self.grid_y = self.grid_y + 1;
        } else if frog_pos_v < robot_pos_v {
            self.grid_y = self.grid_y - 1;
        }
    }

    fn move_by(&mut self, x: isize, y: isize) {
        let new_x = (self.grid_x as isize)+x;
        let new_y = (self.grid_y as isize)+y;
        if new_x >= 0 && new_y >= 0 && new_x < 8 && new_y < 6 {
            self.grid_x = self.grid_x+x as usize;
            self.grid_y = self.grid_y+y as usize;
            self.moved = true;
        }
    }
}

fn sprite_named(name: &'static str) -> Sprite<Texture> {
    let tex = Rc::new(Texture::from_path(&Path::new(format!("./{}.png",name))).unwrap());
    Sprite::from_texture(tex.clone())
}

fn random_grid_position<R: Rng>(random_horizontal: &Range<usize>, random_vertical: &Range<usize>, rng: &mut R) -> (usize, usize) {
    (random_horizontal.ind_sample(rng), random_vertical.ind_sample(rng))
}


fn main() {
    let death_animation = Sequence(vec![
        Action(Blink(1.0, 5)),
        Action(ScaleBy(0.5, 0.0, -0.5)),
        Action(FadeOut(1.0)),
    ]);
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
    let (pos_h, pos_v) = random_grid_position(&random_horizontal, &random_vertical, &mut rng);
    let frog_in_progress = ActorInProgress{ grid_x: pos_h, grid_y: pos_v, alive: true, sprite_name: "frog" };
    let mut frog_texture = sprite_named(frog_in_progress.sprite_name);
    frog_texture.set_position(frog_in_progress.scene_x(), frog_in_progress.scene_y());
    let mut frog = frog_in_progress.to_actor(scene.add_child(frog_texture));

    let mut ref_cell_robots : Vec<RefCell<Actor>> = vec![];
    for _ in 1..5 {
        let (pos_h, pos_v) = random_grid_position(&random_horizontal, &random_vertical, &mut rng);
        let robot_in_progress = ActorInProgress{ grid_x: pos_h, grid_y: pos_v, alive: true, sprite_name: "robot" };
        let mut robot_texture = sprite_named(robot_in_progress.sprite_name);
        robot_texture.set_position(robot_in_progress.scene_x(), robot_in_progress.scene_y());
        let robot = robot_in_progress.to_actor(scene.add_child(robot_texture));
        ref_cell_robots.push(RefCell::new(robot));
    }

    let ref mut gl = GlGraphics::new(opengl);
    let window = RefCell::new(window);

    for e in piston::events(&window) {
        use piston::event::{ PressEvent, RenderEvent };
        scene.event(&e);
        if let Some(args) = e.render_args() {
            use graphics::*;
            gl.draw([0, 0, args.width as i32, args.height as i32], |c, gl| {
                graphics::clear([0.8705882, 0.8039215, 0.690196, 1.0], gl);
                scene.draw(&c, gl);
            });
        }
        for robot_cell in ref_cell_robots.iter() {
            let mut robot = robot_cell.borrow_mut();
            if frog.collides(&robot)&& frog.alive {
                frog.alive = false;
                scene.run(frog.sprite_uuid.clone(), &death_animation);
            }
            for other_robot_cell in ref_cell_robots.iter() {
                if other_robot_cell.borrow_state() == BorrowState::Unused {
                    if robot.alive && robot.collides(other_robot_cell.borrow().deref()) {
                        robot.alive = false;
                        let mut wreck_texture = sprite_named("wreck");
                        wreck_texture.set_position(robot.scene_x(), robot.scene_y());
                        let wreck_uuid = scene.add_child(wreck_texture);
                        scene.run(robot.sprite_uuid.clone(), &death_animation);
                        let seq2 = Sequence(vec![Action(FadeOut(0.0)),Wait(1.5), Action(FadeIn(1.0))]);
                        scene.run(wreck_uuid.clone(), &seq2);
                    }
                }
            }
        }
        if frog.alive {
            if let Some(Button::Keyboard(key)) = e.press_args() {
                frog.moved = false;
                match key {
                    Key::Q => frog.move_by(-1,-1),
                    Key::W => frog.move_by(0,-1),
                    Key::E => frog.move_by(1,-1),
                    Key::A => frog.move_by(-1,0),
                    Key::D => frog.move_by(1,0),
                    Key::Z => frog.move_by(-1,1),
                    Key::X => frog.move_by(0,1),
                    Key::C => frog.move_by(1,1),
                    _ => { },
                }
                if key == Key::T {
                    let (pos_h, pos_v) = random_grid_position(&random_horizontal, &random_vertical, &mut rng);
                    frog.grid_x = pos_h;
                    frog.grid_y = pos_v;
                    scene.run(frog.sprite_uuid.clone(),&Action(MoveTo(0.0, frog.scene_x(), frog.scene_y())));
                }
                if frog.moved {
                    scene.run(frog.sprite_uuid.clone(),&Action(MoveTo(0.5, frog.scene_x(), frog.scene_y())));
                    for robot_cell in ref_cell_robots.iter() {
                        let mut robot = robot_cell.borrow_mut();
                        if robot.alive {
                            robot.move_towards(&frog);
                        }
                        scene.run(robot.sprite_uuid.clone(), &Action(MoveTo(0.75, robot.scene_x(), robot.scene_y())));
                    }
                }
            }
        }
    }
}
