use types::*;
use shaders::*;
use utils::{img_path_to_texture, text_to_texture};
use glium::texture::Texture2d;
use glium::backend::glutin_backend::GlutinFacade;
use oxicloak::{read_file, s_to_usize, s_to_f32};
use glium::index::PrimitiveType;


#[derive(Debug)]
pub enum BuiltInShape {
   Rectangle, Triangle, Circle, TriangleFan
}
#[derive(Clone)]
pub struct Shape {
   pub vertices : Vec<ColorVertex>,
   pub primitive_type : Option<PrimitiveType>

}
#[allow(dead_code)]
impl Shape {
   pub fn new() -> Shape {
      Shape {
         vertices : Vec::new(),
         primitive_type : None
      }
   }
   pub fn add(&mut self, v : ColorVertex) {
      self.vertices.push(v);
   }
   pub fn add_coords(&mut self, x1 : Coord, y1 : Coord,
                     x2 : Coord, y2 : Coord)
   {
      self.vertices.push(ColorVertex::new(x1, y1, x2, y2));
   }
   pub fn print(&self) {
      for vert in &*self.vertices { vert.print(); }
   }

   pub fn new_builtin(shape_type : BuiltInShape) -> Shape {
      let mut shape = Shape::new(); //TODO
      match shape_type {
         BuiltInShape::Triangle => {
            /*shape.add_coords(-0.5, -0.5, 0.0, 0.0);
            shape.add_coords(0.0, 0.5, 0.0, 1.0);
            shape.add_coords(0.5, -0.5, 1.0, 0.0);*/
            shape.add_coords(-0.5, -0.4330127, 0.0, 0.0);
            shape.add_coords(0.0, 0.4330127, 0.0, 1.0);
            shape.add_coords(0.5, -0.4330127, 1.0, 0.0);
         },
         BuiltInShape::Rectangle => {
            shape.add_coords(0.5, -0.5, 1.0, 0.0);
            shape.add_coords(-0.5, -0.5, 0.0, 0.0);
            shape.add_coords(-0.5, 0.5, 0.0, 1.0);
            shape.add_coords(0.5, -0.5, 1.0, 0.0);
            shape.add_coords(0.5, 0.5, 1.0, 1.0);
            shape.add_coords(-0.5, 0.5, 0.0, 1.0);
         },
         BuiltInShape::TriangleFan => {
            /*so far, only have triangle strip, in loscript draw circle
            for rot = pi/4, n = 8
            let x = 0.35355339;
            shape.add_coords(0.0, 0.0, 0.0, 0.0);
            shape.add_coords(-x, -x, 0.0, 0.0);
            shape.add_coords(0.0, -0.5, 0.0, 0.0);*/
            let x = 0.121267813;
            let y = 0.515388203;
            let z = 0.48507;
            //rot = pi/4/2/2, n = 8*2*2
            shape.add_coords(0.0, 0.0, 0.0, 0.0);
            shape.add_coords(-x, -z, 0.0, 0.0);
            shape.add_coords(0.0, -0.5, 0.0, 0.0);

            /*shape.add_coords(0.0, 0.0, 0.0, 0.0);
            shape.add_coords(-0.5, -0.5, 0.0, 0.0);
            shape.add_coords(0.0, -0.5, 0.0, 0.0);*/
         },
         _ => { panic!("not implemented shape: {:?}", shape_type); }
      }
      shape.primitive_type = Some(PrimitiveType::TrianglesList);
      shape
   }
   pub fn new_vertices(vertices_ : Vec<ColorVertex>,
                       primitive_type : PrimitiveType)
   -> Shape
   {
      Shape { vertices : vertices_, primitive_type : Some(primitive_type) }
   }
   pub fn from_obj_file(&mut self, path : &str) {
      let data = read_file(path).unwrap();

      let mut verts = Vec::new();
      let mut square_indices = Vec::new();
      let mut trig_indices = Vec::new();

      let lines = data.split("\n").collect::<Vec<&str>>();
      for line in lines.iter() {
         let words = line.split(" ").collect::<Vec<&str>>();
         //TODO: check words >= 1
         if words[0] == "#" { continue; }
         else if words[0] == "v" {
            if words.len() != 4 {
               println!("bad line: {}", line);
            } else {
               let v = ColorVertex {
                  pos: [s_to_f32(words[1]), s_to_f32(words[2])],
                  tex_pos : [0.0, 0.0]
               };
               verts.push(v);
            }
         } else if words[0] == "f" {
            if words.len() != 5 && words.len() != 4 { println!("bad face: {}", line) }
            else {
               let words_rest = words.split_first().unwrap().1;
               let wordsf = words_rest
                              .iter()
                              .map(|&x| s_to_usize(x) - 1)
                              .collect::<Vec<_>>();

               if words.len() == 5 {
                  square_indices.push((wordsf[0], wordsf[1], wordsf[2], wordsf[3]));
               } else if words.len() == 4 {
                  trig_indices.push((wordsf[0], wordsf[1], wordsf[2]));
               }
            }
         }
         else { /*println!("unknown line type {}", words[0]);*/ }
      }
      //println!("num triangles: {}", trig_indices.len());
      //println!("num rectangles: {}", square_indices.len());
      for index in trig_indices {
         let (p1, p2, p3) = index;
         let (v1, v2, v3) = (verts[p1], verts[p2], verts[p3]);
         self.add(v1);
         self.add(v2);
         self.add(v3);
      }
      for index in square_indices {
         let (p1, p2, p3, p4) = index;
         let (v1, v2, v3, v4) = (verts[p1], verts[p2], verts[p3], verts[p4]);
         self.add(v1);
         self.add(v2);
         self.add(v3);
         self.add(v1);
         self.add(v4);
         self.add(v3);
      }
      self.primitive_type = Some(PrimitiveType::TrianglesList);
   }
   pub fn from_obj_file_uv_texture(&mut self, path : &str) {

      let data = read_file(path).unwrap();

      let mut verts = Vec::new();
      let mut face_indices = Vec::new(); //f vertex_i/face_i v_i/f_i v_i/fi
      let mut vt = Vec::new();

      let lines = data.split("\n").collect::<Vec<&str>>();
      for line in lines.iter() {
         let words = line.split(" ").collect::<Vec<&str>>();

         //TODO: check words >= 1
         match words[0] {
            "#" => continue,
            "vt" => {
               if words.len() != 3 { println!("bad number of args to vt: {}", line); }
               vt.push([s_to_f32(words[1]), s_to_f32(words[2])]);
            },
            "v" => {
               if words.len() != 4 {
                  println!("bad line: {}", line);
               } else {
                  /*let v = ColorVertex {
                     pos: [s_to_f32(words[1]), s_to_f32(words[2])],
                     tex_pos : [0.0, 0.0]
                  };
                  verts.push(v);*/
                  verts.push([s_to_f32(words[1]), s_to_f32(words[2])]);
               }
            },
            "f" => {
               let err_msg = "need to triangulate model before exporting to obj";
               if words.len() != 4 { println!("bad face: {}; {}", line, err_msg); }
               else {

                  let words_rest = words.split_first().unwrap().1;
                  let ti_vi = words_rest
                                 .iter()
                                 .map(|&x| x.split("/")
                                            .map(|y| s_to_usize(y) - 1).collect::<Vec<_>>())
                                 .collect::<Vec<_>>();
                  //vi = index in vt (vertex texture coords)
                  //fi = index in verts (vertex ie coords in screen)
                  //[[vi,fi], [vi,fi], [vi,fi]]
                  //face_indices.push(ti_vi);
                  let v1 = (ti_vi[0][0], ti_vi[0][1]);
                  let v2 = (ti_vi[1][0], ti_vi[1][1]);
                  let v3 = (ti_vi[2][0], ti_vi[2][1]);
                  face_indices.push((v1, v2, v3));
                  //face_indices.push((wordsf[0], wordsf[1], wordsf[2]));
                  //trig_indices.push((wordsf[0], wordsf[1], wordsf[2]));
               }
            },
            _ => {}
         }
      }
      for (i1, i2, i3) in face_indices {
         let (vert_in1, tex_in1) = i1;
         let (vert_in2, tex_in2) = i2;
         let (vert_in3, tex_in3) = i3;

         let vert1 = ColorVertex {
            pos: verts[vert_in1],
            tex_pos : vt[tex_in1]
         };
         self.add(vert1);

         let vert2 = ColorVertex {
            pos: verts[vert_in2],
            tex_pos : vt[tex_in2]
         };
         self.add(vert2);

         let vert3 = ColorVertex {
            pos: verts[vert_in3],
            tex_pos : vt[tex_in3]
         };
         self.add(vert3);

      }
      /*for index in trig_indices {
         let (p1, p2, p3) = index;
         let (v1, v2, v3) = (verts[p1], verts[p2], verts[p3]);
         self.add(v1);
         self.add(v2);
         self.add(v3);
      }*/
      self.primitive_type = Some(PrimitiveType::TrianglesList);
   }
}

#[derive(PartialEq)]
pub enum TextureType {
   None, Image, Color
}

//#[derive(Clone)]
pub struct Model {
   pub shape : Option<Shape>,
   pub texture_type : TextureType,
   pub color : Option<Color>,
   pub img_path : Option<String>,
   pub texture : Option<Texture2d>,
   pub shader_name : Option<String>,
   pub text : Option<String>
}

impl Model {
   pub fn new() -> Model {
      Model {
         shape : None, color : None, img_path : None,
         texture : None, shader_name : None,
         texture_type : TextureType::None,
         text : None
      }
   }
   pub fn text(&mut self, text_ : String) -> &mut Model {
      self.text = Some(text_); self
   }
   pub fn shape(&mut self, shape_ : Shape) -> &mut Model {
      self.shape = Some(shape_); self
   }
   pub fn color(&mut self, color_ : Color) -> &mut Model {
      self.color = Some(color_); self
   }
   pub fn img_path(&mut self, img_path_ : String) -> &mut Model {
      self.img_path = Some(img_path_); self
   }
   //pub fn shader_name(&mut self, shader_name_ : String) -> &mut Model { self.shader_name = Some(shader_name_); self }

   pub fn get_texture(&self) -> &Option<Texture2d> {
      &self.texture
   }
   pub fn finalize(&mut self, sm : &mut ShaderManager, display : &GlutinFacade)
      -> Model
   {
      //calculate missing stuff and make sure we didn't get too many arguments
      let mut texture_type = TextureType::None;

      if let Some(c) = self.color {
         texture_type = TextureType::Color;
         let name = format!("color-{}{}{}{}", c.0, c.1, c.2, c.3);
         self.shader_name = Some(name.to_string());

         if let Some(_) = sm.shaders.get(&*name) {}
         else {
            let frag_sh_src = frag_shader_color(c);
            sm.add_shader(display, name, VERT_SH_COLOR, &frag_sh_src);
         }
      }
      let texture = if let Some(img_path) = self.img_path.clone() {
         if texture_type == TextureType::Color {
            panic!("Cannot have texture image and color");
         }
         texture_type = TextureType::Image;
         self.shader_name = Some("texture".to_string());
         Some(img_path_to_texture(img_path, display))
      } else if let Some(t) = self.text.clone() { //todo: fix texture type and stuff
         texture_type = TextureType::Image;
         //self.shader_name = Some("texture".to_string());
         self.shader_name = Some("text".to_string());
         Some(text_to_texture(t, display))
      } else { None };

      if texture_type == TextureType::None {
         panic!("Model needs to either have color or img_path");
      }

      Model {
         shape: self.shape.clone(), color: self.color,
         img_path: self.img_path.clone(), texture: texture,
         shader_name: self.shader_name.clone(),
         texture_type: texture_type,
         text: self.text.clone()
      }
   }
   //fn draw() {}
}


