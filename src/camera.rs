// Camera structure, providing convenience methods
// for moving and rotating camera in world space (i.e. changing view matrix),
// as well as setting a projection method (perspective or orthographic projections)
// Also allows merging view + projection matrices into one (by multiplication)

use std::cell::{RefCell, Ref, Cell};

pub struct Camera {
   projection_matrix: glam::Mat4,
   view_matrix: glam::Affine3A,
   view_projection_matrix: RefCell<glam::Mat4>,
   is_merged: Cell<bool>,
}

impl Camera {
   pub fn translate_to(&mut self, world_camera_position: impl Into<glam::Vec3A>) -> &mut Self {
      self.view_matrix.translation = world_camera_position.into();
      self.is_merged.set( false);
      self
   }

   pub fn translate(&mut self, view_delta_position: impl Into<glam::Vec3A>) -> &mut Self {
      self.view_matrix.translation -= view_delta_position.into();
      self.is_merged.set( false);
      self
   }

   pub fn look_at(&mut self, look_at: impl Into<glam::Vec3A>) -> &mut Self {
      let forward = (self.view_matrix.translation - look_at.into()).normalize();
      let right = forward.cross(glam::Vec3A::Y);
      let up = right.cross(forward);
      self.view_matrix.matrix3 = glam::mat3a(right, up, forward);
      self.is_merged.set( false);
      self
   }

   pub fn look_forward(&mut self, unit_forward_direction: impl Into<glam::Vec3A>) -> &mut Self {
      let forward = unit_forward_direction.into();
      let right = glam::Vec3A::Y.cross(forward);
      let up = forward.cross(right);
      self.view_matrix.matrix3 = glam::mat3a(right, up, forward);
      self.is_merged.set(false);
      self
   }

   pub fn with_perspective(&mut self, fov_y: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> &mut Self {
      self.projection_matrix = glam::Mat4::perspective_rh_gl(fov_y, aspect_ratio, z_near, z_far);
      self.is_merged.set( false);
      self
   }

   pub fn with_orthographic(&mut self, bounds: glam::Vec3) -> &mut Self {
      self.projection_matrix = glam::Mat4::orthographic_rh_gl(
         -bounds.x, bounds.x,
         -bounds.y, bounds.y,
         -bounds.z, bounds.z);
      self.is_merged.set( false);
      self
   }

   pub fn view(&self) -> &glam::Affine3A {
      &self.view_matrix
   }

   pub fn projection(&self) -> &glam::Mat4 {
      &self.projection_matrix
   }

   pub fn view_projection(&self) -> Ref<glam::Mat4> {
      if !self.is_merged.get()  {
         *self.view_projection_matrix.borrow_mut() = self.projection_matrix * self.view_matrix;
         self.is_merged.set(true);
      }
      self.view_projection_matrix.borrow()
   }

   pub fn position(&self) -> glam::Vec3A {
      self.view_matrix.translation
   }

   pub fn x_axis(&self) -> glam::Vec3A {
      self.view_matrix.x_axis
   }

   pub fn y_axis(&self) -> glam::Vec3A {
      self.view_matrix.y_axis
   }

   pub fn z_axis(&self) -> glam::Vec3A {
      self.view_matrix.z_axis
   }
}

impl Default for Camera {
   fn default() -> Self {
      Self {
         projection_matrix: glam::Mat4::orthographic_rh_gl(
            -1.0, 1.0, -1.0, 1.0, -1.0, 1.0),
         view_matrix: glam::Affine3A::IDENTITY,
         view_projection_matrix: RefCell::new(glam::Mat4::default()),
         is_merged: Cell::new(false),
      }
   }
}