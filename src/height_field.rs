// HeightField - encapsulation of FFT-based algorithm for 
// water height field generation at moment t

use crate::{consts, make_compute_shader};
use crate::wave::Wind;
use glium::texture::{RawImage2d, Texture2dDataSource};
use glium::{Display, Texture2d, uniforms, };
use glium::uniforms::{Sampler};
use glium::GlObject;
extern crate gl;

type TextureResult<T> = Result<T, glium::texture::TextureCreationError>;
pub struct HeightField {
   // size of computing domain on GPU
   // has to be a power of 2 (preferably below 2048)
   size: usize,
   physical_size: f32, // meters

   // waves with smaller length will be discarded (to improve convergence)
   length_cutoff_meters: f32,

   // period of global ocean motion
   period_sec: f32,

   // waves perpendicular to wind will be suppressed
   wind: Wind,

   // 2D FFT twiddle indices (complex exponentials, that are independent of time)
   twiddle_indices: Option<Texture2d>,

   // initial stationary 2D spectrum of height field, that doesn't depend on time
   // one w.r.t. wave magnitude and other is complex conjugate w.r.t. negative wave magnitude
   base_spectrum: Option<Texture2d>,
   base_spectrum_minus_k: Option<Texture2d>,
   spectrum_amplitude: f32,

   // spectrum at time t (defines displacement in all 3 dimensions)
   spectrum_realization_dx: Option<Texture2d>,
   spectrum_realization_dy: Option<Texture2d>,
   spectrum_realization_dz: Option<Texture2d>,

   // to compute FFT in OpenGL, we'll have to compute logN levels
   // of butterfly algorithm, for this we use "ping-pong" approach,
   // where we have two textures, one to read the current level state
   // the other to write newly computed level
   // the textures are swapped after each each level
   height_field_current: Option<Texture2d>,
   height_field_previous: Option<Texture2d>,

   // OpenGL GPU program to combine precomputed maps, and 
   // find height field for the current frame
   fft_compute_shader: glium::program::ComputeShader,
}

impl HeightField {
   pub fn new(display: &Display, lattice_size: usize, physical_size: f32, period_sec: f32) -> Self {
      let mut instance = Self {
         size: lattice_size,
         physical_size,
         length_cutoff_meters: consts::WAVELENGTH_CUTOFF_METERS,
         spectrum_amplitude: consts::PHILLIPS_SPECTRUM_AMPLITUDE,
         period_sec,
         wind: Wind::new(consts::WIND_VELOCITY,
            glam::vec2(consts::WIND_DIRECTION_X, consts::WIND_DIRECTION_Y)),
         twiddle_indices: None,
         base_spectrum: None,
         base_spectrum_minus_k: None,
         spectrum_realization_dx: None,
         spectrum_realization_dy: None,
         spectrum_realization_dz: None,
         height_field_current: None,
         height_field_previous: None,
         fft_compute_shader: make_compute_shader!(display, "shaders/fft.comp"),
      };
      instance.regenerate_textures(display, lattice_size, physical_size);
      instance
   }

   pub fn regenerate_textures(&mut self, display: &Display, size: usize, physical_size: f32) {
      self.size = size;
      self.physical_size = physical_size;
      let twiddle_indices = Self::make_twiddle_indices(display, self.size)
         .expect("Couldn't generate texture for FFT twiddle indices");
      self.twiddle_indices = twiddle_indices.into();
      let (base_spectrum, base_spectrum_minus_k) = Self::make_base_spectrum(
         display, self.size, physical_size, self.spectrum_amplitude,
         self.length_cutoff_meters, &self.wind)
         .expect("Couldn't generate two textures of FFT base spectum");
      self.base_spectrum = Some(base_spectrum);
      self.base_spectrum_minus_k = Some(base_spectrum_minus_k);

      let (spectrum_dx, spectrum_dy, spectrum_dz) = Self::make_spectrum_realizations(display, self.size)
         .expect("Couldn't generate three textures of spectrum time realization");

      self.spectrum_realization_dx = Some(spectrum_dx);
      self.spectrum_realization_dy = Some(spectrum_dy);
      self.spectrum_realization_dz = Some(spectrum_dz);

      // let noise_map = Self::make_noise_map(display, self.size)
      //    .expect("Couldn't generate texture with Standard Normal random values");
      // self.noise_map = Some(noise_map);

      let (field_current, field_previous) = Self::make_height_field(display, self.size)
      .expect("Couldn't generate empty textures for height field");
      self.height_field_current = Some(field_current);
      self.height_field_previous = Some(field_previous);
   }

   // Launch "fft.comp" GPU program, pass input data 
   // (input textures with precomputed stationary spectrum,
   //  output textures for spectrum realization)
   pub fn compute_height_field_gpu(&self, time: f32) {
      unsafe {
         if let glium::Handle::Id(id) = self.fft_compute_shader.get_id() {
            gl::UseProgram(id);

            let location = gl::GetUniformLocation(id, "o_hkt_dx".as_ptr() as *const i8);
            // assert!(location != -1);
            gl::Uniform1i(location, 0);
            gl::BindImageTexture(0, self.spectrum_realization_dx.as_ref().unwrap().get_id(),
               0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RGBA32F);

            let location = gl::GetUniformLocation(id, "o_hkt_dy".as_ptr() as *const i8);
            // assert!(location != -1);
            gl::Uniform1i(location, 1);
            gl::BindImageTexture(1, self.spectrum_realization_dy.as_ref().unwrap().get_id(),
               0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RGBA32F);

            let location = gl::GetUniformLocation(id, "o_hkt_dz".as_ptr() as *const i8);
            // assert!(location != -1);
            gl::Uniform1i(location, 2);
            gl::BindImageTexture(2, self.spectrum_realization_dz.as_ref().unwrap().get_id(),
               0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RGBA32F);

            let location = gl::GetUniformLocation(id, "i_h0k".as_ptr() as *const i8);
            // assert!(location != -1);
            gl::Uniform1i(location, 3);
            gl::BindImageTexture(3, self.base_spectrum.as_ref().unwrap().get_id(),
               0, gl::FALSE, 0, gl::READ_ONLY, gl::RGBA32F);

            let location = gl::GetUniformLocation(id, "i_h0_minus_k".as_ptr() as *const i8);
            // assert!(location != -1);
            gl::Uniform1i(location, 4);
            gl::BindImageTexture(4, self.base_spectrum_minus_k.as_ref().unwrap().get_id(),
               0, gl::FALSE, 0, gl::READ_ONLY, gl::RGBA32F);

            let location = gl::GetUniformLocation(id, "u_PhysicalSize".as_ptr() as *const i8);
            // assert!(location != -1);
            gl::Uniform1f(location, self.physical_size);

            let location = gl::GetUniformLocation(id, "u_BaseFrequency".as_ptr() as *const i8);
            // log::info!("loc {}", location);
            // assert!(location != -1);
            gl::Uniform1f(location, self.base_frequency());

            let location = gl::GetUniformLocation(id, "u_Time".as_ptr() as *const i8);
            // log::info!("loc {} {} ", self.base_frequency(), self.physical_size);
            // assert!(location != -1);
            gl::Uniform1f(location, time);

            gl::DispatchCompute((self.size / 8) as u32, (self.size / 8) as u32, 1);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);

            gl::UseProgram(0);
         }
      }
      // self.fft_compute_shader.execute(glium::uniform!{
      //    // o_hkt_dx: self.spectrum_realization_dx.as_ref().unwrap(),
      //    // o_hkt_dy: self.spectrum_realization_dy.as_ref().unwrap()),
      //    // o_hkt_dz: self.spectrum_realization_dz.as_ref().unwrap()),
      //    // i_h0k : self.base_spectrum.as_ref().unwrap()),
      //    // i_h0_minus_k : self.base_spectrum_minus_k.as_ref().unwrap()),
      //    u_PhysicalSize: self.physical_size,
      //    u_BaseFrequency: self.base_frequency(),
      //    u_Time: time,
      // }, (self.size / 16) as u32, (self.size / 16) as u32, 1);
   }

   pub fn set_period(&mut self, period_sec: f32) {
      self.period_sec = period_sec
   }

   pub fn twiddle_indices_texture(&self) -> Option<&glium::Texture2d> {
      self.twiddle_indices.as_ref()
   }

   pub fn base_spectrum_normal(&self) -> Option<&glium::Texture2d> {
      self.base_spectrum.as_ref()
   }

   pub fn base_spectrum_conjugate(&self) -> Option<&glium::Texture2d> {
      self.base_spectrum_minus_k.as_ref()
   }

   pub fn current_height_field(&self) -> Option<&glium::Texture2d> {
      self.height_field_current.as_ref()
   }

   pub fn previous_height_field(&self) -> Option<&glium::Texture2d> {
      self.height_field_previous.as_ref()
   }

   pub fn spectrum_realization_dx(&self) -> Option<&glium::Texture2d> {
      self.spectrum_realization_dx.as_ref()
   }

   pub fn spectrum_realization_dy(&self) -> Option<&glium::Texture2d> {
      self.spectrum_realization_dy.as_ref()
   }

   pub fn spectrum_realization_dz(&self) -> Option<&glium::Texture2d> {
      self.spectrum_realization_dz.as_ref()
   }

   // To make simulation periodic, we need to make all subwaves frequencies
   // to be a multiple of some base frequency
   pub fn base_frequency(&self) -> f32 {
      2.0 * consts::PI / self.period_sec
   }

   // Radix indices and twiddle factors of FFT algorithm, which can be precomputed
   // Since we compute height field on GPU via OpenGL,
   // those indices and factors should be stored in way accessible by OpenGL.
   // The easiest - is a 2D texture
   // Red+Green channels  - twiddle factors
   // Blue+Alpha channels - indices
   fn make_twiddle_indices(display: &Display, size: usize) -> TextureResult<Texture2d> {
      let n_cols = usize::trailing_zeros(size) as usize; // == log2(size)
      let mut reorder = vec![0; size];
      for col in 0..n_cols {
         let summand = size >> (col+1);
         let checker = 1 << col;
         for row in 0..size {
            if (row / checker) % 2 == 1 {
               reorder[row] += summand;
            }
         }
      }
      // log::info!("{:?}", reorder);
      let mut twiddle_indices_cpu = vec![vec![(0.0f32, 0.00f32, 0.00f32, 0.00f32); n_cols]; size];
      let coef = 2.0 * consts::PI * glam::vec2(0.0, 1.0) / (size as f32);
      for row in 0..size {
         // first column has reversed order of inputs
         {
            let (index, other_index) =
               if row % 2 == 1 {(reorder[row-1], reorder[row])}
               else {(reorder[row], reorder[row+1])};
            twiddle_indices_cpu[row][0] =
               (1.0, 0.0, index as f32, other_index as f32);
         }
         for col in 1..n_cols {
            let two2col = 1 << (col);
            let b = size >> (col + 1);
            let k = (row * b) % size;
            //log::info!("{}", k);
            let mut twiddle = crate::complex::complex_exp(coef * (k as f32));
            let is_bottom_wing = (row % (two2col * 2)) >= two2col;
            let (index, other_index) = if is_bottom_wing {
               // twiddle = -twiddle;
               (row - two2col, row)
            } else {
               (row, row + two2col)
            };
            twiddle_indices_cpu[row][col] =
                  (twiddle.x, twiddle.y, index as f32, other_index as f32);
         }
      }
      glium::Texture2d::with_format(display,
         twiddle_indices_cpu,
         glium::texture::UncompressedFloatFormat::F32F32F32F32,
         glium::texture::MipmapsOption::NoMipmap)
   }

   // Initial Fourier components \hat{h}(k) and conjugate \hat{h}^*(-k)
   // at time t=0 of the waves spectrum, which can be precomputed
   // Since we compute height field on GPU via OpenGL,
   // those should be stored in way accessible by OpenGL.
   // The easiest - is a 2D texture, for each component
   fn make_base_spectrum(display: &glium::Display, size: usize, physical_size: f32, spectrum_amplitude: f32, wave_cutoff: f32, wind: &Wind) -> TextureResult<(Texture2d, Texture2d)> {
      let mut spectrum_cpu = vec![vec![(0.00f32, 0.00f32, 0.00f32, 0.00f32); size]; size];
      let mut spectrum_conjugate_cpu = vec![vec![(0.00f32, 0.00f32, 0.00f32, 0.00f32,); size]; size];

      use rand::Rng;
      let mut rng = rand::thread_rng();

      let inv_sqrt2 = 1.0 / f32::sqrt(2.0);
      for row in 0..size {
         for col in 0..size {
            let k = crate::wave::wavevector_from_coords((row, col), size, physical_size);
            let phillips_sqrt = f32::sqrt(
               Self::phillips_spectrum(spectrum_amplitude, k, wave_cutoff, wind));
            {
               let rnd_real : f32 = rng.sample(rand_distr::StandardNormal);
               let rnd_imag : f32 = rng.sample(rand_distr::StandardNormal);
               let entry = inv_sqrt2 * glam::vec2(rnd_real, rnd_imag) * phillips_sqrt;
               let entry = entry.clamp(glam::vec2(0.0, 0.0), glam::vec2(1000000.0, 1000000.0));
               spectrum_cpu[row][col] = (entry.x, entry.y, 0.0, 1.0);
            }
            {
               let rnd_real : f32 = rng.sample(rand_distr::StandardNormal);
               let rnd_imag : f32 = rng.sample(rand_distr::StandardNormal);
               let entry = inv_sqrt2 * glam::vec2(rnd_real, rnd_imag) * phillips_sqrt;
               let entry = entry.clamp(glam::vec2(0.0, 0.0), glam::vec2(1000000.0, 1000000.0));
               spectrum_conjugate_cpu[row][col] = (entry.x, entry.y, 0.0, 1.0);
            }
         }
      }
      let spectrum = glium::Texture2d::with_format(display,
         spectrum_cpu,
         glium::texture::UncompressedFloatFormat::F32F32F32F32,
         glium::texture::MipmapsOption::NoMipmap);

      let spectrum_conjugate = glium::Texture2d::with_format(display,
         spectrum_conjugate_cpu,
         glium::texture::UncompressedFloatFormat::F32F32F32F32,
         glium::texture::MipmapsOption::NoMipmap);

      spectrum.and_then(|spectrum|
         spectrum_conjugate.and_then(|spectrum_conjugate|
            Ok((spectrum, spectrum_conjugate))))
   }

   // Not used now, but can be used for precomputing the data on GPU
   fn make_noise_map(display: &Display, size: usize) -> TextureResult<Texture2d> {
      let mut rng = rand::thread_rng();
      use rand::prelude::*;
      let noise_map_cpu = (0..size).map(|_|
         (0..size).map(|_|
            (rng.sample(rand_distr::StandardNormal),
             rng.sample(rand_distr::StandardNormal),
             rng.sample(rand_distr::StandardNormal),
             rng.sample(rand_distr::StandardNormal))
         ).collect::<Vec<(f32, f32, f32, f32)>>()
      ).collect::<Vec<_>>();

      glium::Texture2d::with_format(display,
         noise_map_cpu,
         glium::texture::UncompressedFloatFormat::F32F32F32F32,
         glium::texture::MipmapsOption::NoMipmap)
   }
   // Create empty textures, where we'll be writing the results
   // Since FFT is computed in O(logN) stages, we need to read result of 
   // previous stage from one texture, then write result in the other
   // texture, after that the textures are swapped
   fn make_height_field(display: &glium::Display, size: usize) -> TextureResult<(Texture2d, Texture2d)> {
      let f0 = glium::Texture2d::empty_with_format(display,
         glium::texture::UncompressedFloatFormat::F32,
         glium::texture::MipmapsOption::NoMipmap,
      size as u32, size as u32);
      let f1 = glium::Texture2d::empty_with_format(display,
         glium::texture::UncompressedFloatFormat::F32,
         glium::texture::MipmapsOption::NoMipmap,
      size as u32, size as u32);
      f0.and_then(|f0|
         f1.and_then(|f1|
            Ok((f0, f1))))
   }

   // Create empty textures, where spectrum at moment t will be computed (on GPU)
   // The "fft.comp" shader file defines how displacement of vertices in all
   // 3 dimensions is computed
   fn make_spectrum_realizations(display: &glium::Display, size: usize) -> TextureResult<(Texture2d, Texture2d, Texture2d)> {
      let dy = glium::Texture2d::empty_with_format(display,
         glium::texture::UncompressedFloatFormat::F32F32F32F32,
         glium::texture::MipmapsOption::NoMipmap,
         size as u32, size as u32);
      let dx = glium::Texture2d::empty_with_format(display,
         glium::texture::UncompressedFloatFormat::F32F32F32F32,
         glium::texture::MipmapsOption::NoMipmap,
         size as u32, size as u32);
      let dz = glium::Texture2d::empty_with_format(display,
         glium::texture::UncompressedFloatFormat::F32F32F32F32,
         glium::texture::MipmapsOption::NoMipmap,
         size as u32, size as u32);
      dx.and_then(|dx|
         dy.and_then(|dy|
            dz.and_then(|dz|
            Ok((dx, dy, dz)))))
   }

   // The most typical spectrum of oceanic waves, has many researched extensions
   // to improve convergence or impose requirements (like shallow water)
   fn phillips_spectrum(amplitude: f32, wave_vector: glam::Vec2, wave_cutoff: f32, wind: &Wind) -> f32 {
      let k_len = wave_vector.length();
      let k_sqr = k_len * k_len;
      let k_4 = k_sqr * k_sqr;

      let largest_wave_len = wind.largest_wavelength();
      let numerator = f32::exp(-1.0 / (k_sqr * largest_wave_len * largest_wave_len));

      let wind_dot_4 = wave_vector.dot(wind.direction()) / k_len;
      let wind_dot_4 = wind_dot_4 * wind_dot_4;
      let wind_dot_4 = wind_dot_4 * wind_dot_4;

      let small_wave_cutoff = f32::exp(
         -k_sqr * wave_cutoff * wave_cutoff);

      amplitude * numerator * wind_dot_4 * small_wave_cutoff / k_4
   }
}

