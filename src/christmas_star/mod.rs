extern crate gl;
extern crate cgmath;

use gl::types::{GLuint,GLfloat,GLsizeiptr,GLboolean};
use std::ptr;
use std::mem;
use cgmath::{Vector3,Vector4,EuclideanVector};

use glutil;
use game;
use light;
use control;

pub struct ChristmasStar {
    geometry: Geometry,
    resource: GlResource,
    directional: light::Directional,
}

pub struct Parameter<'a> {
    pub fragment_shader_path: &'a str,
    pub vertex_shader_path: &'a str,
}

struct GlResource {
    shader_program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    indice_num : i32,
}

struct Geometry {
    center : cgmath::Vector3<f32>,
    left_canyon_offset : cgmath::Vector3<f32>,
    right_canyon_offset : cgmath::Vector3<f32>,
    long_spike_length  : f32,
    short_spike_length : f32,
    thickness : f32, 
}

struct Vertex {
    position: cgmath::Vector3<f32>,
    normal: cgmath::Vector3<f32>,
    diffuse_color: cgmath::Vector4<f32>,
}

impl Vertex {
    fn new(pos: cgmath::Vector3<f32>, norm: cgmath::Vector3<f32>, diffuse: cgmath::Vector4<f32>) -> Vertex {
        Vertex {
            position : pos,
            normal : norm,
            diffuse_color : diffuse,
        }
    }
}

impl ChristmasStar {
    pub fn new() -> ChristmasStar {
        ChristmasStar{
            geometry : Geometry {
                center : cgmath::Vector3::new(0.0,0.0,0.0),
                left_canyon_offset : cgmath::Vector3::new(0.05, 0.1, 0.0),
                right_canyon_offset : cgmath::Vector3::new(0.1, 0.05, 0.0),
                long_spike_length : 0.8,
                short_spike_length : 0.3,
                thickness : 0.1,
            },
            resource : GlResource {
                shader_program : 0,
                vao: 0,
                vbo: 0,
                indice_num: 0,
            },
            directional : light::Directional::new("direction_to_light".to_string(), cgmath::Vector3::new(0.4, 0.5, 0.5)),
        }
    }

    pub fn init(&mut self, param: Parameter) -> Result<(), String> {
        let vss = match glutil::read_shader(param.vertex_shader_path) {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed reading vertex shader: {}", e)),
        };
        let fss = match glutil::read_shader(param.fragment_shader_path) {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed reading fragment shader: {}", e)),
        };
        let vs = try!(glutil::compile_shader(vss.as_slice(), gl::VERTEX_SHADER));
        let fs = try!(glutil::compile_shader(fss.as_slice(), gl::FRAGMENT_SHADER));
        let prog = try!(glutil::link_program(vs, fs));

        // remove shaders since we've finished linking it
        glutil::remove_shader(prog, vs);
        glutil::remove_shader(prog, fs);
 
        let (vao, vbo, ind_num) = try!(init_buffers(&self.geometry));

        let r = &mut self.resource;
        r.shader_program = prog;
        r.vao = vao;
        r.vbo = vbo;
        r.indice_num = ind_num;

        Ok(())
    }

    pub fn close(&mut self) {
        let r = &mut self.resource;
        unsafe {
            gl::DeleteBuffers(1, &r.vbo);
            gl::DeleteVertexArrays(1, &r.vao);
        }
        glutil::remove_program(r.shader_program);
        r.shader_program = 0;
        r.vbo = 0;
        r.vao = 0;
    }
}

impl game::Object for ChristmasStar {
    fn update(&mut self, cs: &control::State) -> Result<(),String> {
        let delta = 0.01;
        if cs.move_up {
            self.directional.position.y += delta;
        }
        if cs.move_down {
            self.directional.position.y += -delta;
        }
        if cs.move_left {
            self.directional.position.x += -delta;
        }
        if cs.move_right {
            self.directional.position.x += delta;
        }
        // println!("directional: {}", self.directional.position);
        Ok(())
    }

    fn draw(&self) -> Result<(),String> {
        let r = &self.resource;
        unsafe {
            gl::UseProgram(r.shader_program);
            try!(glutil::check_error());

            // update uniform variables if there were any change 
            let cstr = self.directional.name.to_c_str();
            let loc = gl::GetUniformLocation(r.shader_program, cstr.as_ptr());
            try!(glutil::check_error());
            let dir_to_light = self.directional.position.sub(&self.geometry.center);
            gl::Uniform3f(loc, dir_to_light.x, dir_to_light.y, dir_to_light.z);
            try!(glutil::check_error());

            gl::BindVertexArray(r.vao);
            try!(glutil::check_error());
            gl::DrawArrays(gl::TRIANGLES, 0, r.indice_num);
            try!(glutil::check_error());
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
        Ok(())
    }
}

fn calculate_normal(
    v0: &cgmath::Vector3<f32>,
    v1: &cgmath::Vector3<f32>,
    v2: &cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
    let e0 = v1.sub(v0);
    let e1 = v2.sub(v0);
    e0.cross(&e1).normalize()
}

fn add_partial_vertices(
    center: cgmath::Vector3<f32>,
    left_canyon_offset: cgmath::Vector3<f32>,
    right_canyon_offset: cgmath::Vector3<f32>,
    left_long_spike: cgmath::Vector3<f32>,
    right_long_spike: cgmath::Vector3<f32>,
    short_spike: cgmath::Vector3<f32>,
    depth: f32,
    vertices: &mut Vec<Vertex>) {
    let lcox = left_canyon_offset.x;
    let lcoy = left_canyon_offset.y;
    let rcox = right_canyon_offset.x;
    let rcoy = right_canyon_offset.y;
    let cx = center.x;
    let cy = center.y;
    let cz = center.z;
    let llsx = left_long_spike.x;
    let llsy = left_long_spike.y;
    let rlsx = right_long_spike.x;
    let rlsy = right_long_spike.y;
    let ssx = short_spike.x;
    let ssy = short_spike.y;
    let top = cz+depth;
    let c = cgmath::Vector3::new(cx, cy, top);
    let lc = cgmath::Vector3::new(cx+lcox, cy+lcoy, cz);
    let rc = cgmath::Vector3::new(cx+rcox, cy+rcoy, cz);
    let ss = cgmath::Vector3::new(cx+ssx, cy+ssy, cz);
    let ll = cgmath::Vector3::new(cx+llsx, cy+llsy, cz);
    let rl = cgmath::Vector3::new(cx+rlsx, cy+rlsy, cz);
    let n0 = calculate_normal(&c, &lc, &ll);
    // println!("n: {}", n0);
    let diffuse = cgmath::Vector4::new(0.9,0.9,0.0,1.0);
    vertices.push(Vertex::new(c, n0, diffuse));
    vertices.push(Vertex::new(ll, n0, diffuse));
    vertices.push(Vertex::new(lc, n0, diffuse));

    let n1 = calculate_normal(&c, &ss, &lc);
    vertices.push(Vertex::new(c, n1, diffuse));
    vertices.push(Vertex::new(lc, n1, diffuse));
    vertices.push(Vertex::new(ss, n1, diffuse));

    let n2 = calculate_normal(&c, &rc, &ss);
    vertices.push(Vertex::new(c, n2, diffuse));
    vertices.push(Vertex::new(ss, n2, diffuse));
    vertices.push(Vertex::new(rc, n2, diffuse));

    let n3 = calculate_normal(&c, &rl, &rc);
    vertices.push(Vertex::new(c, n3, diffuse));
    vertices.push(Vertex::new(rc, n3, diffuse));
    vertices.push(Vertex::new(rl, n3, diffuse));
}

fn generate_vertices(geom: &Geometry) -> Vec<Vertex> {
    let c = geom.center;
    let ls = geom.long_spike_length;
    let ss = geom.short_spike_length;
    let lco = geom.left_canyon_offset;
    let rco = geom.right_canyon_offset;
    let depth = geom.thickness * 0.5;

    // add a quarter of a star per each add function
    let mut vertices : Vec<Vertex> = Vec::new();
    // top right
    add_partial_vertices(c,
        lco,
        rco, 
        cgmath::Vector3::new(0.0,ls,0.0),
        cgmath::Vector3::new(ls,0.0,0.0),
        cgmath::Vector3::new(ss,ss,0.0),
        depth,
        &mut vertices);
    // bottom right
    add_partial_vertices(c,
        cgmath::Vector3::new(rco.x, -rco.y, lco.z),
        cgmath::Vector3::new(lco.x, -lco.y, rco.z),
        cgmath::Vector3::new(ls,0.0,0.0),
        cgmath::Vector3::new(0.0,-ls,0.0),
        cgmath::Vector3::new(ss,-ss,0.0),
        depth,
        &mut vertices);
    // bottom left
    add_partial_vertices(c,
        cgmath::Vector3::new(-lco.x, -lco.y, lco.z),
        cgmath::Vector3::new(-rco.x, -rco.y, rco.z),
        cgmath::Vector3::new(0.0,-ls,0.0),
        cgmath::Vector3::new(-ls,0.0,0.0),
        cgmath::Vector3::new(-ss,-ss,0.0),
        depth,
        &mut vertices);
    // top left
    add_partial_vertices(c,
        cgmath::Vector3::new(-rco.x, rco.y, lco.z),
        cgmath::Vector3::new(-lco.x, lco.y, rco.z),
        cgmath::Vector3::new(-ls, 0.0, 0.0),
        cgmath::Vector3::new(0.0, ls, 0.0),
        cgmath::Vector3::new(-ss, ss, 0.0),
        depth,
        &mut vertices);
    vertices
}

fn init_buffers(geom : &Geometry) -> Result<(GLuint, GLuint, i32), String> {
    let vertices = generate_vertices(geom);
    let mut vao = 0;
    let mut vbo = 0;
    let mut indice_num = 0;
    unsafe {
        // Create Vertex Array Object
        gl::GenVertexArrays(1, &mut vao);
        try!(glutil::check_error());
        gl::BindVertexArray(vao);
        try!(glutil::check_error());
        // Create a Vertex Buffer Object and copy the vertex data to it
        gl::GenBuffers(1, &mut vbo);
        try!(glutil::check_error());
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        try!(glutil::check_error());
        let vertice_size = mem::size_of::<Vertex>();
        let vertice_num = vertices.len();
        let float_size = mem::size_of::<GLfloat>();
        // println!("Vertex size: {}", vertice_size);
        // println!("Vertex num: {}", vertice_num);
        // println!("float size: {}", float_size);
        gl::BufferData(gl::ARRAY_BUFFER,
            (vertice_num * vertice_size) as GLsizeiptr,
            mem::transmute(&vertices[0]), gl::STATIC_DRAW);
        try!(glutil::check_error());

        indice_num = vertice_num as i32;

        // values taken from layout location in vertex shader
        let pos_location = 0;
        let norm_location = 1;
        let diffuse_location = 2;
        let stride = vertice_size as i32;
        gl::EnableVertexAttribArray(pos_location);
        try!(glutil::check_error());
        gl::EnableVertexAttribArray(norm_location);
        try!(glutil::check_error());
        gl::EnableVertexAttribArray(diffuse_location);
        try!(glutil::check_error());
        gl::VertexAttribPointer(pos_location, 3, gl::FLOAT, gl::FALSE as GLboolean, stride, ptr::null());
        try!(glutil::check_error());
        let normal_offset = mem::transmute(float_size * 3);  // normal comes after position
        gl::VertexAttribPointer(norm_location, 3, gl::FLOAT, gl::FALSE as GLboolean, stride, normal_offset);
        try!(glutil::check_error());
        let diffuse_offset = mem::transmute(float_size * (3+3)); // diffuse comes after position and normal
        gl::VertexAttribPointer(diffuse_location, 4, gl::FLOAT, gl::FALSE as GLboolean, stride, diffuse_offset);
        try!(glutil::check_error());
        gl::BindVertexArray(0);
    }
    Ok((vao, vbo, indice_num))
}

