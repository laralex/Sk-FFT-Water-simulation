use std::sync::Arc;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}

glium::implement_vertex!(Vertex, position);

pub fn make_tri_mesh(display: &glium::Display, size: (u32, u32)) -> (glium::VertexBuffer<Vertex>, glium::index::NoIndices) {
   let (width, height) = size;
   let mut shape = Vec::<Vertex>::with_capacity ((width*height) as usize);

   for y in 0..height {
      let y = y as f32;
      for x in 0..width+1 {
         let x = x as f32;
         shape.push( Vertex { position: [x, y + 1.0] } );
         shape.push( Vertex { position: [x, y] } );
      }
      // degenerate triangles
      shape.push( Vertex { position: [width as f32, y + 1.0] } );
      shape.push( Vertex { position: [0.0, y + 1.0] } );
   }
   let vbo = glium::VertexBuffer::new(display, &shape).unwrap();
   let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
   (vbo, indices)
}

pub fn make_program(display: &glium::Display) -> glium::Program {
   crate::shader::make_program!(display, "shaders/mesh_grid.vert", "shaders/mesh_grid.frag")
}