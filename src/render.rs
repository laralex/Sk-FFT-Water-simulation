use glium::backend::glutin;
use glium::{Frame, uniform, PolygonMode};
use crate::camera::Camera;

pub trait Renderer {
   fn draw_to(&self, frame: &mut Frame, camera: &Camera);
}

#[derive(Copy, Clone, PartialEq)]
pub enum DrawMode {
   Mesh, Wireframe
}
pub struct DrawParametersVariant<'a> {
   mesh_parameters: glium::DrawParameters<'a>,
   wireframe_parameters: glium::DrawParameters<'a>,
   pub current_mode: DrawMode,
}

impl<'a> DrawParametersVariant<'a> {
   pub fn new() -> Self {
      let wireframe_parameters = glium::DrawParameters {
         depth: glium::Depth {
             test: glium::draw_parameters::DepthTest::IfLess,
             write: true,
             .. Default::default()
         },
         backface_culling: glium::BackfaceCullingMode::CullClockwise,
         polygon_mode: PolygonMode::Line,
         line_width: Some(1.5),
         .. Default::default()
      };
      let mut mesh_parameters = wireframe_parameters.clone();
      mesh_parameters.polygon_mode = PolygonMode::Fill;
      Self {
         mesh_parameters, wireframe_parameters, current_mode: DrawMode::Wireframe,
      }
   }

   pub fn current_parameters(&self) -> &glium::DrawParameters<'a> {
      match self.current_mode {
         DrawMode::Mesh => &self.mesh_parameters,
         DrawMode::Wireframe => &self.wireframe_parameters,
      }
   }
}
pub struct WaterRenderer<'a> {
   mesh_grid_program: glium::Program,
   mesh_grid_vertices: glium::VertexBuffer<crate::mesh_grid::Vertex>,
   mesh_grid_indices: glium::index::NoIndices,
   draw_parameters: DrawParametersVariant<'a>,
}

impl<'a> WaterRenderer<'a> {
   pub fn new(display: &glutin::Display, grid_size: (u32, u32), cell_size: f32) -> Self {
      let mesh_grid_program = crate::mesh_grid::make_program(display);
      let (mesh_grid_vertices, mesh_grid_indices)
         = crate::mesh_grid::make_tri_mesh(display, grid_size, cell_size);
      let draw_parameters = DrawParametersVariant::new();
      Self {
         mesh_grid_program,
         mesh_grid_vertices,
         mesh_grid_indices,
         draw_parameters,
      }
   }

   pub fn recreate_mesh_grid(&mut self, display: &glutin::Display, grid_size: (u32, u32), cell_size: f32) {
      let (mesh_grid_vertices, mesh_grid_indices)
         = crate::mesh_grid::make_tri_mesh(display, grid_size, cell_size);
      self.mesh_grid_vertices = mesh_grid_vertices;
      self.mesh_grid_indices = mesh_grid_indices;
   }

   pub fn set_draw_mode(&mut self, draw_mode: DrawMode) {
      self.draw_parameters.current_mode = draw_mode;
   }
}

impl<'a> Renderer for WaterRenderer<'a> {
   fn draw_to(&self, frame: &mut Frame, camera: &Camera) {
      use glium::Surface;
      let uniforms = &uniform! {model_view_projection:
         camera.get_view_projection().to_cols_array_2d()};
      frame.draw(
         &self.mesh_grid_vertices,
         &self.mesh_grid_indices,
         &self.mesh_grid_program,
         uniforms,
         self.draw_parameters.current_parameters(),
      ).unwrap()
   }
}