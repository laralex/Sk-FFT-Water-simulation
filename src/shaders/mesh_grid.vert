#version 330

uniform mat4 model_view_projection;

in vec2 position;
in vec2 tex_coord;

out vec2 v_tex_coord;

void main() {
   gl_Position = model_view_projection * vec4(position, 0.0, 1.0);
   v_tex_coord = tex_coord;
}