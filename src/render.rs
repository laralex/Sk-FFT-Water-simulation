use glium::backend::glutin;
use glium::uniforms::{Sampler, SamplerBehavior};
use glium::{Frame, uniform, PolygonMode};
use crate::camera::Camera;

// All OpenGL rendering wrapped here
// - water mesh
// - debugging textures

// Common interface
pub trait Renderer {
   fn draw_to(&self, frame: &mut Frame, camera: &Camera);
}

// Mesh - fill triangles with texture
// Wireframe - draw only lines
#[derive(Copy, Clone, PartialEq)]
pub enum DrawMode {
   Mesh, Wireframe
}

// ============
// Switcheable settings
// ============
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
         backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
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

// ============
// Water mesh
// ============
pub struct WaterRenderer<'a> {
   mesh_grid_program: glium::Program,
   mesh_grid_vertices: glium::VertexBuffer<crate::mesh_grid::Vertex>,
   mesh_grid_indices: glium::index::NoIndices,
   draw_parameters: DrawParametersVariant<'a>,
   albedo_map: glium::Texture2d,
   mesh_grid_model: glam::Affine3A,
}

impl<'a> WaterRenderer<'a> {
   pub fn new(display: &glutin::Display, grid_size: (u32, u32), facet_size: f32) -> Self {
      let mesh_grid_program = crate::mesh_grid::make_program(display);
      let (mesh_grid_vertices, mesh_grid_indices)
         = crate::mesh_grid::make_tri_mesh(display, grid_size, facet_size);
      let draw_parameters = DrawParametersVariant::new();
      let albedo_map = crate::mesh_grid::make_textures(display);
      let mesh_grid_model = glam::Affine3A::from_translation(
         Self::get_grid_center(grid_size, facet_size).into());
      Self {
         mesh_grid_program,
         mesh_grid_vertices,
         mesh_grid_indices,
         draw_parameters,
         albedo_map,
         mesh_grid_model,
      }
   }

   pub fn recreate_mesh_grid(&mut self, display: &glutin::Display, grid_size: (u32, u32), facet_size: f32) {
      let (mesh_grid_vertices, mesh_grid_indices)
         = crate::mesh_grid::make_tri_mesh(display, grid_size, facet_size);
      self.mesh_grid_vertices = mesh_grid_vertices;
      self.mesh_grid_indices = mesh_grid_indices;
      self.mesh_grid_model.translation = Self::get_grid_center(grid_size, facet_size);
   }

   pub fn get_grid_center(grid_size: (u32, u32), facet_size: f32) -> glam::Vec3A {
      glam::vec3a(grid_size.0 as f32 * facet_size * -0.5,
          0.0, grid_size.1 as f32 * facet_size * -0.5)
   }

   pub fn set_draw_mode(&mut self, draw_mode: DrawMode) {
      self.draw_parameters.current_mode = draw_mode;
   }
}

impl<'a> Renderer for WaterRenderer<'a> {
   fn draw_to(&self, frame: &mut Frame, camera: &Camera) {
      use glium::Surface;
      let albedo_map = Sampler::new(&self.albedo_map);
      albedo_map.anisotropy(8);
      albedo_map.minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapNearest);
      albedo_map.magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear);
      let uniforms = &uniform! {
         model_view_projection: (*camera.view_projection() * self.mesh_grid_model).to_cols_array_2d(),
         albedo_map: albedo_map,
      };
      frame.draw(
         &self.mesh_grid_vertices,
         &self.mesh_grid_indices,
         &self.mesh_grid_program,
         uniforms,
         self.draw_parameters.current_parameters(),
      ).unwrap()
   }
}

// ============
// Drawing debug textures
// ============
pub struct TextureBlitter<'a> {
   texture: Option<&'a glium::Texture2d>,
   blit_rectangle: glium::BlitTarget,
}

impl<'a> TextureBlitter<'a> {
   pub fn new(left: u32, bottom: u32, width: u32, height: u32) -> Self {
      Self {
         blit_rectangle: glium::BlitTarget{
            left, bottom,
            width: width as i32, height: height as i32},
         texture: None,
      }
   }

   pub fn set_texture(&mut self, texture: Option<&'a glium::Texture2d>) {
      self.texture = texture;
   }
}

impl<'a> Renderer for TextureBlitter<'a> {
   fn draw_to(&self, frame: &mut Frame, camera: &Camera) {
      if let Some(texture) = self.texture {
        use glium::Surface;
        texture.as_surface().blit_whole_color_to(frame, &self.blit_rectangle,
      glium::uniforms::MagnifySamplerFilter::Nearest);
      }

   }
}