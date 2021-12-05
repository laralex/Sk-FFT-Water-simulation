use std::sync::Arc;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}

glium::implement_vertex!(Vertex, position);

pub fn make_tri_mesh(display: &glium::Display, size: (u32, u32), cell_size: f32) -> (glium::VertexBuffer<Vertex>, glium::index::NoIndices) {
   let (width, height) = size;
   let mut shape = Vec::<Vertex>::with_capacity ((width*height) as usize);

   let mut fy = 0.0;
   for _ in 0..height {
      let mut fx = 0.0;
      for _ in 0..width+1 {
         shape.push( Vertex { position: [fx, fy + cell_size] } );
         shape.push( Vertex { position: [fx, fy] } );
         fx += cell_size;
      }
      // degenerate triangles
      shape.push( Vertex { position: [(width as f32)*cell_size, fy + cell_size] } );
      shape.push( Vertex { position: [0.0, fy + cell_size] } );
      fy += cell_size;
   }
   let vbo = glium::VertexBuffer::new(display, &shape).unwrap();
   let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
   (vbo, indices)
}

pub fn make_program(display: &glium::Display) -> glium::Program {
   crate::shader::make_program!(display, "shaders/mesh_grid.vert", "shaders/mesh_grid.frag")
}