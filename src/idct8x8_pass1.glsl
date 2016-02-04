#version 140

// we want to truncate toward zero, but normal rshift truncates to -inf
#define UNBIASED_RSHIFT1(a) (((a) - ((a) >> 31)) >> 1)

// 1D iDCT takes a row and outputs a column
void idct8(out int x[8], const int y[8]) {
  int t0;
  int t1;
  int t1h;
  int t2;
  int t3;
  int t4;
  int t4h;
  int t5;
  int t6;
  int t6h;
  int t7;
  t0 = y[0];
  t1 = y[1];
  t2 = y[2];
  t3 = y[3];
  t4 = y[4];
  t5 = y[5];
  t6 = y[6];
  t7 = y[7];
  t5 -= (t3*2485 + 4096) >> 13;
  t3 += (t5*18205 + 16384) >> 15;
  t5 -= (t3*2485 + 4096) >> 13;
  t7 -= (t1*3227 + 16384) >> 15;
  t1 += (t7*6393 + 16384) >> 15;
  t7 -= (t1*3227 + 16384) >> 15;
  t1 += t3;
  t1h = UNBIASED_RSHIFT1(t1);
  t3 = t1h - t3;
  t5 += t7;
  t7 = UNBIASED_RSHIFT1(t5) - t7;
  t3 += (t5*7489 + 4096) >> 13;
  t5 -= (t3*11585 + 8192) >> 14;
  t3 -= (t5*19195 + 16384) >> 15;
  t6 += (t2*21895 + 16384) >> 15;
  t2 -= (t6*15137 + 8192) >> 14;
  t6 += (t2*21895 + 16384) >> 15;
  t0 += (t4*13573 + 16384) >> 15;
  t4 -= (t0*11585 + 8192) >> 14;
  t0 += (t4*13573 + 16384) >> 15;
  t4 = t2 - t4;
  t4h = UNBIASED_RSHIFT1(t4);
  t2 = t4h - t2;
  t6 = t0 - t6;
  t6h = UNBIASED_RSHIFT1(t6);
  t0 -= t6h;
  t7 = t6h - t7;
  t6 -= t7;
  t2 += UNBIASED_RSHIFT1(t3);
  t3 = t2 - t3;
  t5 += t4h;
  t4 -= t5;
  t0 += t1h;
  t1 = t0 - t1;
  x[0] = t0;
  x[1] = t4;
  x[2] = t2;
  x[3] = t6;
  x[4] = t7;
  x[5] = t3;
  x[6] = t5;
  x[7] = t1;
}

in vec2 v_tex_coords;
out ivec4 col_top;
out ivec4 col_bot;

uniform ivec2 plane_dims;
uniform isampler2D coeffs;

void main() {
  int i;
  int x[8], y[8];

  // find our block
  ivec2 i_tex_coords = ivec2(v_tex_coords * plane_dims);
  ivec2 block = i_tex_coords >> 3;

  // fetch our row of texels in our block
  for (i = 0; i < 8; i++) {
    // we have to shift up for headroom in the transform
    y[i] = texelFetch(coeffs, ivec2((block.x << 3) + i, i_tex_coords.y), 0).r << 4;
  }

  // transform
  idct8(x, y);

  // stuff the column into our output colors
  col_top = ivec4(x[0], x[1], x[2], x[3]);
  col_bot = ivec4(x[4], x[5], x[6], x[7]);
}
