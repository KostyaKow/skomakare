#![feature(type_ascription)]
#[macro_use]
extern crate glium;
extern crate image;
extern crate lambda_oxide;

use model::*;
use shaders::*;
use types::*;
use scene::*;
use utils::*;
use camera::Camera;

use glium::backend::glutin_backend::GlutinFacade;

mod types;
mod shaders;
mod model;
mod scene;
mod camera;
mod utils;

implement_vertex!(ColorVertex, pos, tex_pos);

struct Game {
   display : Display,
   root : Scene,
   shader_manager : ShaderManager,
   cam : Camera,
   clear_color : Color,
   //since we can't store game objects in lisp, we use this to keep track of objects
   script_obj_id : i64
}
impl Game {
   fn new() -> Game {
      use glium::{DisplayBuild, Surface};
      use glium::glutin::WindowBuilder;

      let display_ = WindowBuilder::new().build_glium().unwrap();
      let mut game = Game {
         display : display_,
         cam : Camera::new(),
         root : Scene::new(),
         shader_manager : ShaderManager::new(),
         clear_color : (0.0, 0.0, 1.0, 1.0), //blue
         script_obj_id : 0
      };
      game.shader_manager.add_defaults(&game.display);
      game
   }
   fn draw(&self) {
      use glium::Surface;
      //self.root.draw();
      let init_m = self.cam.get_m();
      let mut target = self.display.draw();
      let cc = self.clear_color;
      target.clear_color(cc.0, cc.1, cc.2, cc.3);

      for game_obj in &self.root.items {
         let obj_m = game_obj.cam.get_m();
         //let final_m = mul_matrices(&init_m, &obj_m);
         //let final_m = obj_m;
         let final_m = mul_matrices(&obj_m, &init_m);

         if let GameObjectType::Model(ref m) = game_obj.data {
            let shape = m.shape.clone().unwrap();

            //draw(&shape, "data/opengl.png");
            use glium::VertexBuffer as VB;
            let vert_buff = VB::new(&self.display,
                                    &shape.vertices)
                                   .unwrap();

            use glium::index::{NoIndices, PrimitiveType};
            let indices = NoIndices(PrimitiveType::TrianglesList);
            let ref shaders = self.shader_manager.shaders;
            let shader_name = m.shader_name.clone().unwrap();
            let program = shaders.get(&*shader_name).unwrap();

            match m.texture_type {
               TextureType::Image => {
                  let t = match *m.get_texture() {
                     Some(ref x) => x, None => panic!("z")
                  };
                  let u = uniform! {
                     matrix: final_m,
                     tex: t
                  };

                  target.draw(&vert_buff, &indices, program, &u,
                              &Default::default()).unwrap();

               },
               TextureType::Color => {
                  let u = uniform! { matrix : final_m };
                  target.draw(&vert_buff, &indices, program, &u,
                              &Default::default()).unwrap();
               }
               _ => { panic!("unknown texture type"); }
            };

         } else { panic!("unsupported object"); }
         /*match game_obj.data {
            Model(ref m) => {
               let shape = m.shape.clone();
               draw(&shape.unwrap(), "data/opengl.png");
            }
            _ => panic!("unsupported object")
         };*/
      }
      target.finish().unwrap();
      use glium::glutin::Event;
      for ev in self.display.poll_events() {
         match ev {
            Event::Closed => panic!("exiting application"), //return,
            _ => ()
         }
      }
   }
}

fn draw(m : &Shape, img_path : &str) {
   use glium::{DisplayBuild, Surface};
   let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();

   //let image = img_path_to_image(img_path);
   //let texture = Texture2d::new(&display, image).unwrap();
   let texture = img_path_to_texture(img_path, &display);

   let vertex_buffer = glium::VertexBuffer::new(&display, &m.vertices).unwrap();
   let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

   use glium::Program;
   let program = Program::from_source(&display, VERT_SH_TEXTURE, FRAG_SH_TEXTURE, None).unwrap();

   let mut i = 0.0f32;
   let mut t = -0.5;

   loop {
      i += 2.0*/*f32::consts::PI*/3.1415/1000.0;
      t += 0.0002;
      if t > 0.5 { t = -0.5; }

      let mut target = display.draw();
      target.clear_color(0.0, 0.0, 1.0, 1.0);

      let uniforms = uniform! {
         matrix: [
            [i.cos(), 0.0,  i.sin(),  0.0],
            [0.0,  1.0, 0.0,  0.0],
            [-i.sin(),  0.0,  i.cos(), 0.0],
            [0.0,  0.0,  0.0,  1.0f32]
            /*[1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [ t , 0.0, 0.0, 1.0f32],*/
         ],
         tex: &texture
      };

      //target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
      //target.draw(&vertex_buffer, &indices, &program, &uniform! { matrix: matrix }, &Default::default()).unwrap();
      target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();

      target.finish().unwrap();

      for ev in display.poll_events() {
         match ev {
            glium::glutin::Event::Closed => return,
            _ => ()
         }
      }
   }
}

#[allow(dead_code)]
fn main_very_old() {
   use glium::{DisplayBuild, Surface};
   let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();

   let mut m = Shape::new();
   m.add_coords(-0.5, -0.5, 0.0, 0.0);
   m.add_coords(0.0, 0.5, 0.0, 1.0);
   m.add_coords(0.5, -0.25, 1.0, 0.0);
   draw(&m, "data/opengl.png");
}

fn engine_main() {
   let mut game = Game::new();

   let shape = Shape::new_builtin(BuiltInShape::Triangle);
   let red = (1.0, 0.0, 0.0, 1.0);

   let m = Model::new()
            .shape(shape).color(red) //.img_path("data/opengl.png")
            .finalize(&mut game.shader_manager, &game.display);

   let triangle = GameObject::new(GameObjectType::Model(m));
   game.root.items.push(triangle);

   use std::time::Duration;
   use std::time::SystemTime;
   let mut start = SystemTime::now();
   let one_tenth_sec = 100000000;
   let mut moved = false;
   let mut rotd = false;

   loop {
      let elapsed = start.elapsed().unwrap().as_secs();
      if elapsed > 1 && !moved {
         game.root.items[0].cam.translate(&[1.0, 0.0]);
         moved = true;
      }
      //if elapsed > 2 && !rotd {game.root.items[0].cam.rotate(90.0); rotd = true;}
      game.draw();
   }
   //draw(&m.shape.unwrap(), "data/opengl.png");
}

use std::sync::mpsc::{Sender, Receiver, channel};
use lambda_oxide::types::{Sexps};
use lambda_oxide::main::Env;
use std::cell::RefCell;

type ObjId = u32;
type CmdSender = Sender<RenderCmd>;
//type CmdReceiver = Receiver<ObjId>;
type CmdReceiver = Receiver<RenderCmd>;

//Obj(<triangle|square|circle>, <color|pic_path>)
enum RenderCmd {
   Obj(String, String), Move(ObjId, Point), Rotate(ObjId, Coord),
   Scale(ObjId, Point), Exit
}

//TODO: add this to core of LambdaOxide
//struct ExtractedArgs { strings : Vec<Strings>, floats : Vec<f64>, exps : Vec<Sexps> }
//arg_extract(args : Vec<Sexps>, format : Vec<String>) -> ExtractedArgs;

fn arg_extract_str(args : Vec<Sexps>, index : usize) -> Option<String> {
   if let Sexps::Str(s) = args[index] {
      Some(s)
   } else { None }
}

fn setup_game_script_env(game : &Game, sender : &CmdSender) -> RefCell<Env> {
   use lambda_oxide::main::{Callable, Root};
   use lambda_oxide::types::{Sexps, arg_extractor, EnvId, err};

   let env = lambda_oxide::main::setup_env();
   //shape(<triangle|square|circle>, <color|pic_path>)

   //let sender_ptr = sender as *const i32;
   //let game_ptr = game as *const i32;

   let shape_ = move |args_ : Sexps, root : Root, table : EnvId| -> Sexps {
      let args = arg_extractor(&args_).unwrap();
      if args.len() < 2 { return err("shape needs 2 arguments"); }

      let shape_type = arg_extract_str(args, 0).unwrap(); //TODO: check types
      let color_type = arg_extract_str(args, 1).unwrap(); //and notify user if wrong
      let cmd = RenderCmd::Obj(shape_type, color_type);

      //kkerr sender.send(cmd).unwrap();
      //kkerr game.script_obj_id += 1;
      //kkerr let ret = Sexps::Int(game.script_obj_id);
      let ret = Sexps::Int(0); //kkerr replace

      ret
   };
   env.borrow_mut().table_add(0, "shape", Callable::BuiltIn(0, Box::new(shape_)));

   let halt_ = move |args : Sexps, root : Root, table : EnvId| -> Sexps {
      let cmd = RenderCmd::Exit;
      //kkerr sender.send(cmd).unwrap();
      err("halting")
   };
   env.borrow_mut().table_add(0, "exit", Callable::BuiltIn(0, Box::new(halt_)));

   env
}

fn main() {
   let mut game = Game::new();

   //use std::sync::mpsc;
   use std::thread::Builder;

   let (tx, rx) : (CmdSender, CmdReceiver) = channel();

   let child = Builder::new().stack_size(8*32*1024*1024).spawn(move || {
      use lambda_oxide::main;

      let env = setup_game_script_env(&game, &tx);
      main::interpreter(Some(env));
   }).unwrap();

   //engine_main();
   loop {
      let script_cmd_res = rx.try_recv();
      if let Ok(script_cmd) = script_cmd_res {
         use RenderCmd::*;

         match script_cmd {
            Obj(shape_type, color_or_texture) => {
               let model_builder = Model::new();

               let shape = match &*shape_type {
                  "triangle" => Shape::new_builtin(BuiltInShape::Triangle),
                  _ => panic!("unsuported shape")
               };
               model_builder.shape(shape);

               let color_opt = match &*color_or_texture {
                  "red"    => Some((1.0, 0.0, 0.0, 1.0)),
                  "green"  => Some((0.0, 1.0, 0.0, 1.0)),
                  "blue"   => Some((0.0, 0.0, 1.0, 1.0)),
                  _        => None
               };
               if let Some(color) = color_opt {
                  model_builder.color(color);
               } else {
                  model_builder.img_path(&*color_or_texture);
               }
               let model = model_builder.finalize(&mut game.shader_manager, &game.display);
               let game_object = GameObject::new(GameObjectType::Model(model));
               game.root.items.push(game_object);
            },
            Exit => break,
            _ => panic!("unsuported command")
         }
      }
      game.draw();
   }
   child.join().unwrap();
}

