use crate::consts::{PI, G};

// Convenience methods for some physical oceanographic relations

pub struct Wind {
   velocity: f32,
   direction: glam::Vec2,
}

impl Wind {
   pub fn new(velocity: f32, direction: glam::Vec2) -> Self {
      Self {
         velocity, direction: direction.normalize(),
      }
   }

   // The largest wave length that can be generated by this wind
   pub fn largest_wavelength(&self) -> f32 {
      self.velocity*self.velocity / G
   }

   pub fn direction(&self) -> glam::Vec2 { self.direction }
   pub fn velocity(&self) -> f32 { self.velocity }
}

// To perform FFT, [0..N] range of lattice coordinates has to be remapped
pub fn wavevector_from_coords((row, col): (usize, usize), lattice_size: usize, physical_size: f32) -> glam::Vec2 {
   2.0 * PI / physical_size *
      glam::vec2(row as f32, col as f32)
}

// From oceanographic reserach it's known that in simplest case,
// wave frequency depends on wavelength as \omega^2 = g*||wave_vector||
pub fn dispersion_frequency(wavevector_magnitude: f32) -> f32 {
   f32::sqrt(G * wavevector_magnitude)
}

// In shallow water, additional an multiplier for dispersion frequency kicks in
pub fn dispersion_frequency_shallow(wavevector_magnitude: f32, water_depth: f32) -> f32 {
   f32::sqrt(G * wavevector_magnitude
      * f32::tanh(wavevector_magnitude*water_depth))
}

// Since default dispersion frequency is continuous, it's hard to
// combine waves in such a way, so that the ocean movement has a certain period
// It can be fixed by ensuring all frequencies are multples of some base frequency
pub fn discrete_dispersion_frequency(dispersion_freq: f32, base_frequency: f32) -> f32 {
   (dispersion_freq / base_frequency).trunc() * base_frequency
}