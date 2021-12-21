// HeightField - encapsulation of FFT-based algorithm for 
// water height field generation at moment t

use crate::wave::Wave;
use crate::consts;
use glium::{Display, Texture2d};

type TextureResult<T> = Result<T, glium::texture::TextureCreationError>;

pub struct HeightField {
   // size of computing domain on GPU
   // has to be a power of 2 (preferably below 2048)
   size: u32,
   // waves comprising an ocean flow
   waves: Vec<Wave>,
   // waves with smaller length will be discarded (to improve convergence)
   length_cutoff_meters: f32,
   // period of global ocean motion
   period_sec: f32,
   // 2D FFT twiddle indices (complex exponentials, that are independent of time)
   twiddle_indices: Texture2d,
   // initial 2D spectrum of height field, that doesn't depend on time
   // one w.r.t. wave magnitude and other is complex conjugate w.r.t. negative wave magnitude
   base_spectrum: (Texture2d, Texture2d),
}

impl HeightField {
   fn new(display: &Display, one_side_size: u32, period_sec: f32) -> Self {
      let twiddle_indices = Self::make_twiddle_indices(display, one_side_size)
         .expect("Couldn't generate texture for FFT twiddle indices");
      let base_spectrum = Self::make_base_spectrum(display, one_side_size)
         .expect("Couldn't generate two textures of FFT base spectum");
      Self {
         size: one_side_size,
         waves: vec![],
         length_cutoff_meters: consts::WAVELENGTH_CUTOFF_METERS,
         period_sec,
         twiddle_indices,
         base_spectrum,
      }
   }

   // To make simulation periodic, we need to make all subwaves frequencies
   // to be a multiple of some base frequency
   fn base_frequency(&self) -> f32 {
      2.0 * consts::PI / self.period_sec
   }

   // Radix indices of FFT algorithm, which can be precomputed
   // Since we compute height field on GPU via OpenGL,
   // those indices should be stored in way accessible by OpenGL.
   // The easiest - is a 2D texture
   pub fn make_twiddle_indices(display: &Display, one_side_size: u32) -> TextureResult<Texture2d> {
      glium::Texture2d::empty_with_format(display,
         glium::texture::UncompressedFloatFormat::U16U16U16U16,
         glium::texture::MipmapsOption::NoMipmap,
         one_side_size, one_side_size)
   }

   // Initial Fourier components \hat{h}(k) and conjugate \hat{h}^*(-k)
   // at time t=0 of the waves spectrum, which can be precomputed
   // Since we compute height field on GPU via OpenGL,
   // those should be stored in way accessible by OpenGL.
   // The easiest - is a 2D texture, for each component
   pub fn make_base_spectrum(display: &glium::Display, one_side_size: u32) -> TextureResult<(Texture2d, Texture2d)> {
      let spectrum = glium::Texture2d::empty_with_format(display,
         glium::texture::UncompressedFloatFormat::U16U16U16U16,
         glium::texture::MipmapsOption::NoMipmap,
         one_side_size, one_side_size);
      let spectrum_conjugate = glium::Texture2d::empty_with_format(display,
         glium::texture::UncompressedFloatFormat::U16U16U16U16,
         glium::texture::MipmapsOption::NoMipmap,
         one_side_size, one_side_size);
      spectrum.and_then(|spectrum|
         spectrum_conjugate.and_then(|spectrum_conjugate|
            Ok((spectrum, spectrum_conjugate))))
   }
}

