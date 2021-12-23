// HeightField - encapsulation of FFT-based algorithm for 
// water height field generation at moment t

use crate::wave::Wave;
use crate::consts;
use glium::{Display, Texture2d, texture::PixelValue};

type TextureResult<T> = Result<T, glium::texture::TextureCreationError>;

pub struct HeightField {
   // size of computing domain on GPU
   // has to be a power of 2 (preferably below 2048)
   size: usize,
   // waves comprising an ocean flow
   waves: Vec<Wave>,
   // waves with smaller length will be discarded (to improve convergence)
   length_cutoff_meters: f32,
   // period of global ocean motion
   period_sec: f32,
   // 2D FFT twiddle indices (complex exponentials, that are independent of time)
   twiddle_indices: Option<Texture2d>,
   // initial 2D spectrum of height field, that doesn't depend on time
   // one w.r.t. wave magnitude and other is complex conjugate w.r.t. negative wave magnitude
   base_spectrum: Option<Texture2d>,
   base_spectrum_conjugate: Option<Texture2d>,

   // to compute FFT in OpenGL, we'll have to compute logN levels
   // of butterfly algorithm, for this we use "ping-pong" approach,
   // where we have two textures, one to read the current level state
   // the other to write newly computed level
   // the textures are swapped after each each level
   height_field_current: Option<Texture2d>,
   height_field_previous: Option<Texture2d>,
}

impl HeightField {
   pub fn new(display: &Display, one_side_size: usize, period_sec: f32) -> Self {
      let mut instance = Self {
         size: one_side_size,
         waves: vec![],
         length_cutoff_meters: consts::WAVELENGTH_CUTOFF_METERS,
         period_sec,
         twiddle_indices: None,
         base_spectrum: None,
         base_spectrum_conjugate: None,
         height_field_current: None,
         height_field_previous: None,

      };
      instance.regenerate_textures(display, instance.size);
      instance
   }

   pub fn regenerate_textures(&mut self, display: &Display, size: usize) {
      self.size = size;
      let twiddle_indices = Self::make_twiddle_indices(display, self.size)
         .expect("Couldn't generate texture for FFT twiddle indices");
      self.twiddle_indices = twiddle_indices.into();
      let (spectrum, spectrum_conj) = Self::make_base_spectrum(display, self.size)
         .expect("Couldn't generate two textures of FFT base spectum");
      self.base_spectrum = Some(spectrum);
      self.base_spectrum_conjugate = Some(spectrum_conj);

      let (field_current, field_previous) = Self::make_height_field(display, self.size)
      .expect("Couldn't generate empty textures for height field");
      self.height_field_current = Some(field_current);
      self.height_field_previous = Some(field_previous);
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
      self.base_spectrum_conjugate.as_ref()
   }

   pub fn current_height_field(&self) -> Option<&glium::Texture2d> {
      self.height_field_current.as_ref()
   }

   pub fn previous_height_field(&self) -> Option<&glium::Texture2d> {
      self.height_field_previous.as_ref()
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
      log::info!("{:?}", reorder);
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
   fn make_base_spectrum(display: &glium::Display, size: usize) -> TextureResult<(Texture2d, Texture2d)> {
      let spectrum = glium::Texture2d::empty_with_format(display,
         glium::texture::UncompressedFloatFormat::U16U16U16U16,
         glium::texture::MipmapsOption::NoMipmap,
         size as u32, size as u32);
      let spectrum_conjugate = glium::Texture2d::empty_with_format(display,
         glium::texture::UncompressedFloatFormat::U16U16U16U16,
         glium::texture::MipmapsOption::NoMipmap,
         size as u32, size as u32);
      spectrum.and_then(|spectrum|
         spectrum_conjugate.and_then(|spectrum_conjugate|
            Ok((spectrum, spectrum_conjugate))))
   }

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
}

