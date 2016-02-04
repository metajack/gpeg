#version 140

in vec2 v_tex_coords;

out vec3 color;

uniform ivec2 plane_dims;
uniform isampler2D y_plane;
uniform isampler2D cb_plane;
uniform isampler2D cr_plane;

void main() {
  ivec2 y_tex_coords = ivec2(v_tex_coords * plane_dims);
  ivec2 c_tex_coords = y_tex_coords >> 2;

  float y = float(texelFetch(y_plane, y_tex_coords, 0));
  float cb = float(texelFetch(cb_plane, c_tex_coords, 0));
  float cr = float(texelFetch(cr_plane, c_tex_coords, 0));

  float r = y + 128 + 1.402 * cr;
  float g = y + 128 - 0.34414 * cb - 0.71414 * cr;
  float b = y + 128 + 1.772 * cb;

  color = vec3(r / 255, g / 255, b / 255);
}
