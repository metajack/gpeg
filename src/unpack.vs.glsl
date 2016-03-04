#version 140

in vec2 position;
in uint pack_index;

out float f_pack_index;
out vec2 v_tex_coords;

uniform ivec2 plane_dims;

void main() {
  gl_Position = vec4(position, 0.0, 1.0);
  f_pack_index = float(pack_index);
  v_tex_coords = position / float(plane_dims);
}
