#version 330

precision highp float;

uniform sampler2D albedo_map;

in vec2 v_tex_coord;

out vec4 o_color;

void main() {
   o_color = texture(albedo_map, v_tex_coord);
}