#version 140

in vec2 v_tex_coords;

out int color;

uniform ivec2 plane_dims;
uniform usampler2D index_texture;
uniform usampler2D packed_coeffs;

int ZIGZAG[64] = int[64](
   0,  1,  5,  6, 14, 15, 27, 28,
   2,  4,  7, 13, 16, 26, 29, 42,
   3,  8, 12, 17, 25, 30, 41, 43,
   9, 11, 18, 24, 31, 40, 44, 53,
  10, 19, 23, 32, 39, 45, 52, 54,
  20, 22, 33, 38, 46, 51, 55, 60,
  21, 34, 37, 47, 50, 56, 59, 61,
  35, 36, 48, 49, 57, 58, 62, 63
);

void main() {
  // find our block and offset
  ivec2 i_tex_coords = ivec2(v_tex_coords * plane_dims);
  ivec2 block = i_tex_coords >> 3;
  ivec2 offset = i_tex_coords % 8;
  int linear_offset = offset.y * 8 + offset.x;
  int zigzag_offset = ZIGZAG[linear_offset];

  // grab the index where our block's coefficients start
  int index = int(texelFetch(index_texture, block, 0).r);

  int c;
  int pos = -1;
  int turns = 0;
  while (turns < 64) {
    ivec2 cidx = ivec2(index % 512, index >> 9);
    uint packed_coeff = texelFetch(packed_coeffs, cidx, 0).r;

    if (packed_coeff == uint(0)) {
      // all the rest of the coefficients are zero
      c = 0;
      break;
    }
    
    int zeros = int((packed_coeff >> 12) & uint(0x000f));
    int coeff;
    if ((packed_coeff & uint(0x800)) == uint(0x800)) {
      coeff = int(uint(~0x0fff) | (packed_coeff & uint(0x0fff)));
    } else {
      coeff = int(packed_coeff & uint(0x0fff));
    }

    pos += zeros + 1;
      
    if (zigzag_offset < pos) {
      c = 0;
      break;
    }
    if (zigzag_offset == pos) {
      c = coeff;
      break;
    }
    
    index += 1;
    turns += 1;
  }
  
  color = c;
}
