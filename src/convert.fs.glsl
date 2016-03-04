#version 140

in vec2 v_tex_coords;

out vec3 color;

uniform ivec2 plane_dims;
uniform isampler2D y_plane;
uniform isampler2D cb_plane;
uniform isampler2D cr_plane;

void main() {
  ivec2 y_tex_coords = ivec2(v_tex_coords * plane_dims);
  // one shift for decimated color plane, and one for the data being on only
  // one quadrant of the texture
  ivec2 c_tex_coords = y_tex_coords >> 2;

  float y = float(texelFetch(y_plane, y_tex_coords, 0).r) + 128;
  float cb = float(texelFetch(cb_plane, c_tex_coords, 0).r);
  float cr = float(texelFetch(cr_plane, c_tex_coords, 0).r);

  float r = y + 1.402 * cr;
  float g = y - 0.34414 * cb - 0.71414 * cr;
  float b = y + 1.772 * cb;

  color = vec3(r / 255, g / 255, b / 255);
}
