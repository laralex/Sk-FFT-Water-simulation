use std::sync::Arc;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}

glium::implement_vertex!(Vertex, position);

pub fn make_tri_mesh(display: &glium::Display, size: (usize, usize)) -> (glium::VertexBuffer<Vertex>, glium::index::NoIndices) {
   let (width, height) = size;
   let mut shape = Vec::<Vertex>::with_capacity (width*height);
   for row in 0..height-1{
      let row = row as f32;
      for col in 0..width {
         let col = col as f32;
         shape.push( Vertex { position: [row, col] } );
         shape.push( Vertex { position: [row + 1.0, col] } );
      }
      // degenerate triangles
      shape.push( Vertex { position: [row, (width-1) as f32] } );
      shape.push( Vertex { position: [row, 0.0] } );
      shape.push( Vertex { position: [row, 0.0] } );
   }
   let vbo = glium::VertexBuffer::new(display, &shape).unwrap();
   let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
   (vbo, indices)
}

pub fn make_program(display: &glium::Display) -> glium::Program {
   crate::program::make_program!(display, "shaders/mesh_grid.vert", "shaders/mesh_grid.frag")
}