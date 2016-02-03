#version 140

in vec2 v_tex_coords;

out int color;

uniform isampler2D pack_top;
uniform isampler2D pack_bot;

void main() {
  int c;
  ivec2 i_tex_coords = ivec2(int(v_tex_coords.x * 8.0), int(v_tex_coords.y * 8.0));
  if (i_tex_coords.y < 4) {
    c = texelFetch(pack_top, ivec2(i_tex_coords.x, i_tex_coords.x), 0)[i_tex_coords.y];
  } else {
    c = texelFetch(pack_bot, ivec2(i_tex_coords.x, i_tex_coords.x), 0)[i_tex_coords.y - 4];
  }
  color = c >> 4;
}
