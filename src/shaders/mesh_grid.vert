#version 330

uniform mat4 model_view_projection;

in vec2 position;
out vec2 v_position;

void main() {
   gl_Position = model_view_projection * vec4(position, 0.0, 1.0);
   v_position = fract(position);
}