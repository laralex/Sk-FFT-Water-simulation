// Convenience functions for complex numbers calculations

// computing exp{real + i*img}
pub fn complex_exp(complex: glam::Vec2) -> glam::Vec2 {
   let exp_real = f32::exp(complex.x);
   let cos = f32::cos(complex.y);
   let sin = f32::sin(complex.y);
	glam::vec2(cos, sin) * exp_real
}
