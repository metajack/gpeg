#version 140

in vec2 v_tex_coords;
out vec4 color;
uniform isampler2D tex;

void main() {
  ivec2 i_tex_coords = ivec2(int(v_tex_coords.x * 8.0), int(v_tex_coords.y * 8.0));
  ivec4 texel = texelFetch(tex, i_tex_coords, 0);
  float r = (float(texel.r) + 128.0) / 256.0;
  color = vec4(r, r, r, 1.0);
}
