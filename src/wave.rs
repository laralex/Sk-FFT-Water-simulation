use crate::consts::{PI, G};

pub struct Wave {
   // stores direction of wave and magnitude, proportional to wave frequency
   wave_vector: glam::Vec2,
   wave_vector_magnitude: f32,
}

impl Wave {
   pub fn from_lattice_coords(lattice_coords: glam::Vec2, lattice_size_meters: glam::Vec2) -> Self {
      let wave_vector = 2.0*PI*lattice_coords/lattice_size_meters;
      Self {
         wave_vector,
         wave_vector_magnitude: wave_vector.length(),
      }
   }
   
   // Horizontal wave direction
   pub fn wave_vector(&self) -> &glam::Vec2 {
      &self.wave_vector
   }

   // From oceanographic reserach it's known that in simplest case,
   // wave frequency depends on wavelength as \omega^2 = g*||wave_vector||
   pub fn dispersion_frequency(&self) -> f32 {
      f32::sqrt(G * &self.wave_vector_magnitude)
   }

   // In shallow water, additional an multiplier for dispersion frequency kicks in
   pub fn dispersion_frequency_shallow(&self, water_depth: f32) -> f32 {
      f32::sqrt(G * &self.wave_vector_magnitude
         * f32::tanh(&self.wave_vector_magnitude*water_depth))
   }

   // Since default dispersion frequency is continuous, it's hard to
   // combine waves in such a way, so that the ocean movement has a certain period
   // It can be fixed by ensuring all frequencies are multples of some base frequency
   pub fn discrete_dispersion_frequency(&self, base_frequency: f32) -> f32 {
      (self.dispersion_frequency() / base_frequency).trunc() * base_frequency
   }
}