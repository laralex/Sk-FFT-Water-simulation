// Fragment shader program runs for every pixel on a rendered image

// This particular shader takes texture coordinates (u, v) that are barycantrically
// interpolated between 3 vertices of a triangle

// Using those texture coordinates we get color value for this pixel
// from a water texture (see ../textures folder)

#version 330

precision highp float;

uniform sampler2D albedo_map;

in vec2 v_tex_coord;

out vec4 o_color;

void main() {
   o_color = texture(albedo_map, v_tex_coord);
}