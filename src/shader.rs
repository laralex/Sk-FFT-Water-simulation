// Creates a glium::Program object from the given files,
// this object represents an OpenGL program for GPU,
// i.e. a combination of Vertex, Fragment, Geometry shaders
// Example usage:
// make_program!("shaders/mesh_grid.vert", "shaders/mesh_grid.frag")

#[macro_export]
macro_rules! make_program {
   // Construct from Vertex and Fragment shader files
   (
      $display: expr,
      $vertex_shader_file: literal,
      $fragment_shader_file: literal
   ) => {
      glium::Program::from_source($display,
         include_str!($vertex_shader_file),
         include_str!($fragment_shader_file),
         None
      ).expect("Failed to compile OpenGL program")
   };

   // Construct from Vertex, Fragment and Geometry shader files
   (
      $display: expr,
      $vertex_shader_file: literal,
      $fragment_shader_file: literal,
      $geometry_shader_file: literal
   ) => {
      glium::Program::from_source($display,
         include_str!($vertex_shader_file),
         include_str!($fragment_shader_file),
         include_str!($geometry_shader_file)
      ).expect("Failed to compile OpenGL program")
   }
}

pub(crate) use make_program;