#version 330 core
#define num_circles 4
in vec4 gl_FragCoord;
out vec4 Color;
uniform float iTime;
void main()
{
    vec2 uv = gl_FragCoord.xy/800.0;
    vec4 fragColor = vec4( 0.0 );
    Color = vec4( 0.0 );
    for( int idx=0; idx<num_circles; idx++ ) 
    {
        float dist = distance(
            vec2(
                sin(iTime*(0.1672+float(idx)*0.132))*(0.146+float(idx)*0.0132),
                sin(iTime+iTime*(0.221+float(idx)*0.1822))*(0.1131+float(idx)*0.0112)
            ) + .5,
            uv
        );
        Color += vec4( vec3( sin( 100.0*dist ), sin( 110.0*dist ), sin( 120.0*dist ) ) * max( .0, (1.0-dist*3.0) ), .0 );
    }    
}