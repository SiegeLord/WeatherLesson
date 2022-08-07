uniform sampler2D al_tex;
varying vec4 varying_color;
varying vec2 varying_texcoord;
uniform vec2 bitmap_dims;
uniform float swirl_amount;

// http://rastergrid.com/blog/2010/09/efficient-gaussian-blur-with-linear-sampling/
void main()
{
	vec2 center = vec2(0.5, 0.5);
	vec2 uv = varying_texcoord - center;
	float radius = length(uv);
	float angle = atan(uv.y, uv.x);
	angle += swirl_amount * smoothstep(0.2, 0., radius / 2.);
	gl_FragColor = varying_color * texture2D(al_tex, center + vec2(radius * cos(angle), radius * sin(angle)));
}
