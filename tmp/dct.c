#include <stdio.h>
#include <stdint.h>

#define OD_UNBIASED_RSHIFT32(_a, _b) \
 (((int32_t)(((uint32_t)(_a) >> (32 - (_b))) + (_a))) >> (_b))

#define OD_DCT_RSHIFT(_a, _b) OD_UNBIASED_RSHIFT32(_a, _b)

#define OD_DCT_OVERFLOW_CHECK(val, scale, offset, idx) 

void od_bin_fdct8(int32_t y[8], const int32_t *x, int xstride) {
  /*31 adds, 5 shifts, 15 "muls".*/
  /*The minimum theoretical number of multiplies for a uniformly-scaled 8-point
     transform is 11, but the best I've been able to come up with for a
     reversible version with orthonormal scaling is 15.
    We pick up 3 multiplies when computing the DC, since we have an odd number
     of summation stages, leaving no chance to cancel the asymmetry in the last
     one.
    Instead, we have to implement it as a rotation by \frac{\pi}{4} using
     lifting steps.
    We pick up one more multiply when computing the Type IV DCT in the odd
     half.
    This comes from using 3 lifting steps to implement another rotation by
     \frac{\pi}{4} (with asymmetrically scaled inputs and outputs) instead of
     simply scaling two values by \sqrt{2}.*/
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
  /*Initial permutation:*/
  t0 = *(x + 0*xstride);
  t4 = *(x + 1*xstride);
  t2 = *(x + 2*xstride);
  t6 = *(x + 3*xstride);
  t7 = *(x + 4*xstride);
  t3 = *(x + 5*xstride);
  t5 = *(x + 6*xstride);
  t1 = *(x + 7*xstride);
  /*+1/-1 butterflies:*/
  t1 = t0 - t1;
  t1h = OD_DCT_RSHIFT(t1, 1);
  t0 -= t1h;
  t4 += t5;
  t4h = OD_DCT_RSHIFT(t4, 1);
  t5 -= t4h;
  t3 = t2 - t3;
  t2 -= OD_DCT_RSHIFT(t3, 1);
  t6 += t7;
  t6h = OD_DCT_RSHIFT(t6, 1);
  t7 = t6h - t7;
  /*+ Embedded 4-point type-II DCT.*/
  t0 += t6h;
  t6 = t0 - t6;
  t2 = t4h - t2;
  t4 = t2 - t4;
  /*|-+ Embedded 2-point type-II DCT.*/
  /*13573/32768 ~= \sqrt{2} - 1 ~= 0.41421356237309504880168872420970*/
  OD_DCT_OVERFLOW_CHECK(t4, 13573, 16384, 3);
  t0 -= (t4*13573 + 16384) >> 15;
  /*11585/16384 ~= \sqrt{\frac{1}{2}} ~= 0.70710678118654752440084436210485*/
  OD_DCT_OVERFLOW_CHECK(t0, 11585, 8192, 4);
  t4 += (t0*11585 + 8192) >> 14;
  /*13573/32768 ~= \sqrt{2} - 1 ~= 0.41421356237309504880168872420970*/
  OD_DCT_OVERFLOW_CHECK(t4, 13573, 16384, 5);
  t0 -= (t4*13573 + 16384) >> 15;
  /*|-+ Embedded 2-point type-IV DST.*/
  /*21895/32768 ~= \frac{1 - cos(\frac{3\pi}{8})}{\sin(\frac{3\pi}{8})} ~=
     0.66817863791929891999775768652308*/
  OD_DCT_OVERFLOW_CHECK(t2, 21895, 16384, 6);
  t6 -= (t2*21895 + 16384) >> 15;
  /*15137/16384~=sin(\frac{3\pi}{8})~=0.92387953251128675612818318939679*/
  OD_DCT_OVERFLOW_CHECK(t6, 15137, 8192, 7);
  t2 += (t6*15137 + 8192) >> 14;
  /*21895/32768 ~= \frac{1 - cos(\frac{3\pi}{8})}{\sin(\frac{3\pi}{8})}~=
     0.66817863791929891999775768652308*/
  OD_DCT_OVERFLOW_CHECK(t2, 21895, 16384, 8);
  t6 -= (t2*21895 + 16384) >> 15;
  /*+ Embedded 4-point type-IV DST.*/
  /*19195/32768 ~= 2 - \sqrt{2} ~= 0.58578643762690495119831127579030*/
  OD_DCT_OVERFLOW_CHECK(t5, 19195, 16384, 9);
  t3 += (t5*19195 + 16384) >> 15;
  /*11585/16384 ~= \sqrt{\frac{1}{2}} ~= 0.70710678118654752440084436210485*/
  OD_DCT_OVERFLOW_CHECK(t3, 11585, 8192, 10);
  t5 += (t3*11585 + 8192) >> 14;
  /*7489/8192 ~= \sqrt{2}-\frac{1}{2} ~= 0.91421356237309504880168872420970*/
  OD_DCT_OVERFLOW_CHECK(t5, 7489, 4096, 11);
  t3 -= (t5*7489 + 4096) >> 13;
  t7 = OD_DCT_RSHIFT(t5, 1) - t7;
  t5 -= t7;
  t3 = t1h - t3;
  t1 -= t3;
  /*3227/32768 ~= \frac{1 - cos(\frac{\pi}{16})}{sin(\frac{\pi}{16})} ~=
     0.098491403357164253077197521291327*/
  OD_DCT_OVERFLOW_CHECK(t1, 3227, 16384, 12);
  t7 += (t1*3227 + 16384) >> 15;
  /*6393/32768 ~= sin(\frac{\pi}{16}) ~= 0.19509032201612826784828486847702*/
  OD_DCT_OVERFLOW_CHECK(t7, 6393, 16384, 13);
  t1 -= (t7*6393 + 16384) >> 15;
  /*3227/32768 ~= \frac{1 - cos(\frac{\pi}{16})}{sin(\frac{\pi}{16})} ~=
     0.098491403357164253077197521291327*/
  OD_DCT_OVERFLOW_CHECK(t1, 3227, 16384, 14);
  t7 += (t1*3227 + 16384) >> 15;
  /*2485/8192 ~= \frac{1 - cos(\frac{3\pi}{16})}{sin(\frac{3\pi}{16})} ~=
     0.30334668360734239167588394694130*/
  OD_DCT_OVERFLOW_CHECK(t3, 2485, 4096, 15);
  t5 += (t3*2485 + 4096) >> 13;
  /*18205/32768 ~= sin(\frac{3\pi}{16}) ~= 0.55557023301960222474283081394853*/
  OD_DCT_OVERFLOW_CHECK(t5, 18205, 16384, 16);
  t3 -= (t5*18205 + 16384) >> 15;
  /*2485/8192 ~= \frac{1 - cos(\frac{3\pi}{16})}{sin(\frac{3\pi}{16})} ~=
     0.30334668360734239167588394694130*/
  OD_DCT_OVERFLOW_CHECK(t3, 2485, 4096, 17);
  t5 += (t3*2485 + 4096) >> 13;
  y[0] = (int32_t)t0;
  y[1] = (int32_t)t1;
  y[2] = (int32_t)t2;
  y[3] = (int32_t)t3;
  y[4] = (int32_t)t4;
  y[5] = (int32_t)t5;
  y[6] = (int32_t)t6;
  y[7] = (int32_t)t7;
}

void od_bin_idct8(int32_t *x, int xstride, const int32_t y[16]) {
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
  t1h = OD_DCT_RSHIFT(t1, 1);
  t3 = t1h - t3;
  t5 += t7;
  t7 = OD_DCT_RSHIFT(t5, 1) - t7;
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
  t4h = OD_DCT_RSHIFT(t4, 1);
  t2 = t4h - t2;
  t6 = t0 - t6;
  t6h = OD_DCT_RSHIFT(t6, 1);
  t0 -= t6h;
  t7 = t6h - t7;
  t6 -= t7;
  t2 += OD_DCT_RSHIFT(t3, 1);
  t3 = t2 - t3;
  t5 += t4h;
  t4 -= t5;
  t0 += t1h;
  t1 = t0 - t1;
  *(x + 0*xstride) = (int32_t)t0;
  *(x + 1*xstride) = (int32_t)t4;
  *(x + 2*xstride) = (int32_t)t2;
  *(x + 3*xstride) = (int32_t)t6;
  *(x + 4*xstride) = (int32_t)t7;
  *(x + 5*xstride) = (int32_t)t3;
  *(x + 6*xstride) = (int32_t)t5;
  *(x + 7*xstride) = (int32_t)t1;
}

int main() {
  uint8_t pixels[64] = {208, 174, 79, 67, 10, 76, 176, 181,
                        196, 70, 116, 116, 128, 154, 78, 146,
                        164, 163, 225, 214, 213, 205, 105, 143,
                        156, 182, 218, 206, 202, 170, 87, 135,
                        165, 161, 117, 152, 142, 60, 63, 101, 
                        194, 183, 200, 201, 162, 160, 95, 144,
                        210, 165, 188, 198, 126, 129, 97, 208,
                        215, 180, 135, 184, 121, 34, 137, 227};
  
  /* 156, 156, 158, 160, 158, 156, 158, 160, */
  /*                       157, 157, 157, 157, 157, 157, 157, 157, */
  /*                       160, 159, 156, 154, 156, 159, 156, 154, */
  /*                       159, 158, 156, 154, 156, 159, 156, 154, */
  /*                       158, 158, 157, 156, 157, 159, 157, 156, */
  /*                       156, 156, 157, 157, 156, 156, 156, 157, */
  /*                       155, 155, 157, 158, 155, 154, 155, 158, */
  /*                       156, 156, 157, 157, 155, 155, 155, 157}; */

  int32_t scaled_pixels[64];
  int i, x, y;
  int32_t z[64];
  int32_t coeffs[64];
  int32_t output_pixels[64];

  for (i = 0; i < 64; i++) {
    scaled_pixels[i] = (pixels[i] - 128) << 4;
  }

  printf("PIXELS:\n");
  for (y = 0; y < 8; y++) {
    for (x = 0; x < 8; x++) {
      printf("%d, ", pixels[y*8 + x] - 128);
    }
    printf("\n");
  }

  /* forward transform pixels */
  for (i = 0; i < 8; i++) od_bin_fdct8(z + 8*i, scaled_pixels + i, 8);
  for (i = 0; i < 8; i++) od_bin_fdct8(coeffs + 8*i, z + i, 8);

  printf("COEFFS:\n");
  for (y = 0; y < 8; y++) {
    for (x = 0; x < 8; x++) {
      printf("%d, ", coeffs[y*8 + x]);
    }
    printf("\n");
  }

  /* inverse transform coeffs */
  for (i = 0; i < 8; i++) od_bin_idct8(z + i, 8, coeffs + 8*i);

  printf("AFTER FIRST PASS:\n");
  for (y = 0; y < 8; y++) {
    for (x = 0; x < 8; x++) {
      printf("%d, ", z[y*8 + x]);
    }
    printf("\n");
  }

  for (i = 0; i < 8; i++) od_bin_idct8(output_pixels + i, 8, z + 8*i);

  printf("OUTPUT:\n");
  for (y = 0; y < 8; y++) {
    for (x = 0; x < 8; x++) {
      printf("%d ", output_pixels[y*8 + x] >> 4);
    }
    printf("\n");
  }

  return 0;
}
