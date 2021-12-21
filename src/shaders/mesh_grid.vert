// Vertex shader program runs for vertex in a mesh

// This particular shader takes 2D position (x,y) on a flat plane
// and texture coordinate of the current vertex
// (those values are passed from calling CPU code)

// The shader then defines a 3D position of the vertex in the coordinates space
// of camera, which is equivalent to multyplying a 3D local position by 
// model matrix - 4x4 matrix that places the current mesh in a world coordinate space
// view matrix - 4x4 matrix that transforms world coordinates to camera coordinate space
// projection matrix - 4x4 matrix, that maps homogeneous 4D points to image coordinates

#version 330

uniform mat4 model_view_projection;

in vec2 position;
in vec2 tex_coord;

out vec2 v_tex_coord;

void main() {
   gl_Position = model_view_projection * vec4(position.x, 0.0, position.y, 1.0);
   v_tex_coord = tex_coord;
}