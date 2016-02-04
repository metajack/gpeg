#version 140

in vec2 v_tex_coords;

out vec3 color;

uniform sampler2D tex;

void main() {
  color = texture(tex, v_tex_coords).rgb;
}
