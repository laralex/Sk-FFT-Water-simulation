use std::sync::Arc;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
}

glium::implement_vertex!(Vertex, position, tex_coord);

pub fn make_tri_mesh(display: &glium::Display, size: (u32, u32), cell_size: f32) -> (glium::VertexBuffer<Vertex>, glium::index::NoIndices) {
   let (width, height) = size;
   let mut shape = Vec::<Vertex>::with_capacity ((width*height) as usize);
   let tex_size_x = cell_size;
   let tex_size_y = cell_size;
   let mut fy = 0.0;
   let mut tex_y = 0.0;
   for _ in 0..height {
      let mut fx = 0.0;
      let mut tex_x = 0.0;
      for _ in 0..width+1 {
         shape.push( Vertex { position: [fx, fy + cell_size], tex_coord: [tex_x, tex_y + tex_size_y] } );
         shape.push( Vertex { position: [fx, fy], tex_coord: [tex_x, tex_y] } );
         fx += cell_size;
         tex_x += tex_size_x;
      }
      // degenerate triangles
      shape.push( Vertex { position: [(width as f32)*cell_size, fy + cell_size], tex_coord: [0.0, 0.0] } );
      shape.push( Vertex { position: [0.0, fy + cell_size], tex_coord: [0.0, 0.0]} );
      fy += cell_size;
      tex_y += tex_size_y;
   }
   let vbo = glium::VertexBuffer::new(display, &shape).unwrap();
   let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);
   (vbo, indices)
}

pub fn make_program(display: &glium::Display) -> glium::Program {
   crate::shader::make_program!(display, "shaders/mesh_grid.vert", "shaders/mesh_grid.frag")
}

pub fn make_textures(display: &glium::Display) -> glium::Texture2d {
   use std::io::Cursor;
   let image = image::load(Cursor::new(&include_bytes!("textures/seamless_water_1024_1024.jpg")),
                           image::ImageFormat::Jpeg).unwrap().to_rgba8();
   let image_dimensions = image.dimensions();
   let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
   glium::Texture2d::with_mipmaps(display, image, glium::texture::MipmapsOption::AutoGeneratedMipmaps).unwrap()
}