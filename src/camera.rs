use std::cell::{RefCell, Ref};

pub struct Camera {
   projection_matrix: glam::Mat4,
   view_matrix: glam::Affine3A,
   view_projection_matrix: RefCell<glam::Mat4>,
   is_merged: bool,
}

impl Camera {
   pub fn translate_to(&mut self, camera_position: impl Into<glam::Vec3A>) -> &mut Self {
      self.view_matrix.translation = camera_position.into();
      self.is_merged = false;
      self
   }

   pub fn translate_to_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
      self.translate_to(glam::vec3a(x, y, z))
   }

   pub fn translate(&mut self, camera_delta_position: impl Into<glam::Vec3A>) -> &mut Self {
      self.view_matrix.translation += camera_delta_position.into();
      self.is_merged = false;
      self
   }

   pub fn look_at(&mut self, look_at: impl Into<glam::Vec3A>) -> &mut Self {
      let camera_position = self.view_matrix.translation;
      let forward = (camera_position - look_at.into()).normalize();
      let right = glam::Vec3A::Y.cross(forward);
      let up = forward.cross(right);
      self.view_matrix.matrix3 = glam::mat3a(right, up, -forward);
      self.is_merged = false;
      self
   }

   pub fn perspective(&mut self, fov_y: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> &mut Self {
      self.projection_matrix = glam::Mat4::perspective_rh_gl(fov_y, aspect_ratio, z_near, z_far);
      self.is_merged = false;
      self
   }

   pub fn orthographic(&mut self, bounds: glam::Vec3) -> &mut Self {
      self.projection_matrix = glam::Mat4::orthographic_rh_gl(
         -bounds.x, bounds.x,
         -bounds.y, bounds.y,
         -bounds.z, bounds.z);
      self.is_merged = false;
      self
   }

   pub fn get_view(&self) -> &glam::Affine3A {
      &self.view_matrix
   }

   pub fn get_projection(&self) -> &glam::Mat4 {
      &self.projection_matrix
   }

   pub fn get_view_projection(&self) -> Ref<glam::Mat4> {
      if !self.is_merged {
         *self.view_projection_matrix.borrow_mut() = self.projection_matrix * self.view_matrix;
      }
      self.view_projection_matrix.borrow()
   }
}

impl Default for Camera {
   fn default() -> Self {
      Self {
         projection_matrix: glam::Mat4::orthographic_rh_gl(
            -1.0, 1.0, -1.0, 1.0, -1.0, 1.0),
         view_matrix: glam::Affine3A::IDENTITY,
         view_projection_matrix: RefCell::new(glam::Mat4::default()),
         is_merged: false,
      }
   }
}