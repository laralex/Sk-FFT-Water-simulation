#version 330

in vec2 v_position;

out vec4 o_color;

void main() {
   vec2 pos = v_position - 0.5;
   float dist_squared = dot(pos, pos);
   o_color = (dist_squared < 0.5)
          ? vec4(1.0)
          : vec4(vec3(0.0), 1.0);
}