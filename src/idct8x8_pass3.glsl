#version 140

in vec2 v_tex_coords;

out int color;

uniform ivec2 plane_dims;
uniform isampler2D pass2_top;
uniform isampler2D pass2_bot;

void main() {
  int c;

  // find our block and offset
  ivec2 i_tex_coords = ivec2(v_tex_coords * plane_dims);
  ivec2 block = i_tex_coords >> 3;
  ivec2 offset = i_tex_coords % 8;

  // unpack the pixel value
  if (offset.y < 4) {
    c = texelFetch(pass2_top, ivec2(i_tex_coords.x, (block.y << 3) + offset.x), 0)[offset.y];
  } else {
    c = texelFetch(pass2_bot, ivec2(i_tex_coords.x, (block.y << 3) + offset.x), 0)[offset.y - 4];
  }

  // shift back down post transform
  color = c >> 4;
}
