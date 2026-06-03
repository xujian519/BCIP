import { useRef, useMemo } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import * as THREE from 'three';
import React from 'react';

// Vertex shader — simple pass-through with UV
const vertexShader = `
  varying vec2 vUv;
  void main() {
    vUv = uv;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  }
`;

// Fragment shader — animated gradient blobs
const fragmentShader = `
  uniform float uTime;
  uniform vec2 uResolution;
  varying vec2 vUv;

  // Simplex noise helper
  vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
  vec2 mod289(vec2 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
  vec3 permute(vec3 x) { return mod289(((x*34.0)+1.0)*x); }

  float snoise(vec2 v) {
    const vec4 C = vec4(0.211324865405187, 0.366025403784439,
                       -0.577350269189626, 0.024390243902439);
    vec2 i  = floor(v + dot(v, C.yy));
    vec2 x0 = v -   i + dot(i, C.xx);
    vec2 i1;
    i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
    vec4 x12 = x0.xyxy + C.xxzz;
    x12.xy -= i1;
    i = mod289(i);
    vec3 p = permute(permute(i.y + vec3(0.0, i1.y, 1.0))
                             + i.x + vec3(0.0, i1.x, 1.0));
    vec3 m = max(0.5 - vec3(dot(x0,x0), dot(x12.xy,x12.xy),
                            dot(x12.zw,x12.zw)), 0.0);
    m = m*m;
    m = m*m;
    vec3 x_ = 2.0 * fract(p * C.www) - 1.0;
    vec3 h = abs(x_) - 0.5;
    vec3 ox = floor(x_ + 0.5);
    vec3 a0 = x_ - ox;
    m *= 1.79284291400159 - 0.85373472095314 * (a0*a0 + h*h);
    vec3 g;
    g.x  = a0.x  * x0.x  + h.x  * x0.y;
    g.yz = a0.yz * x12.xz + h.yz * x12.yw;
    return 130.0 * dot(m, g);
  }

  void main() {
    vec2 uv = vUv;
    float t = uTime * 0.15;

    // Boot / 品牌渐变 —— 冷灰 + 专利绿（与主界面一致）
    vec3 color1 = vec3(0.290, 0.486, 0.435); // #4A7C6F
    vec3 color2 = vec3(0.420, 0.420, 0.439); // #6B6B70 cool gray
    vec3 color3 = vec3(0.961, 0.961, 0.969); // #F5F5F7
    vec3 color4 = vec3(0.227, 0.545, 0.545); // #3A8B8C cyan

    // Create 3 drifting blobs with noise
    float n1 = snoise(vec2(uv.x * 1.5 + t * 0.3, uv.y * 1.2 - t * 0.2));
    float n2 = snoise(vec2(uv.x * 1.2 - t * 0.25, uv.y * 1.8 + t * 0.15) + 50.0);
    float n3 = snoise(vec2(uv.x * 2.0 + t * 0.2, uv.y * 1.5 - t * 0.3) + 100.0);

    // Normalize and soften
    float blob1 = smoothstep(-0.3, 0.8, n1) * 0.5;
    float blob2 = smoothstep(-0.2, 0.7, n2) * 0.4;
    float blob3 = smoothstep(-0.4, 0.6, n3) * 0.35;

    // Mix colors based on blob influence
    vec3 color = color3; // start with cool light gray
    color = mix(color, color1, blob1);
    color = mix(color, color4, blob2 * 0.6);
    color = mix(color, color2, blob3 * 0.25);

    // Overall ambient glow
    float ambientGlow = 0.15 + 0.1 * sin(t * 0.5);
    color = mix(vec3(1.0, 1.0, 1.0), color, 0.3 + ambientGlow);

    // Very subtle warm vignette
    float vignette = 1.0 - 0.1 * length(uv - 0.5);
    color *= vignette;

    gl_FragColor = vec4(color, 1.0);
  }
`;

interface GradientMeshProps {
  reducedMotion: boolean;
}

function GradientMesh({ reducedMotion }: GradientMeshProps) {
  const meshRef = useRef<THREE.Mesh>(null);
  const materialRef = useRef<THREE.ShaderMaterial>(null);

  const uniforms = useMemo(
    () => ({
      uTime: { value: 0 },
      uResolution: { value: new THREE.Vector2(window.innerWidth, window.innerHeight) },
    }),
    []
  );

  useFrame((state) => {
    if (materialRef.current && !reducedMotion) {
      materialRef.current.uniforms.uTime.value = state.clock.elapsedTime;
    }
  });

  return (
    <mesh ref={meshRef}>
      <planeGeometry args={[2, 2]} />
      <shaderMaterial
        ref={materialRef}
        vertexShader={vertexShader}
        fragmentShader={fragmentShader}
        uniforms={uniforms}
      />
    </mesh>
  );
}

// Check reduced motion preference
const prefersReducedMotion =
  typeof window !== 'undefined' &&
  window.matchMedia('(prefers-reduced-motion: reduce)').matches;

const MeshGradient: React.FC = () => {
  return (
    <div style={{ width: '100%', height: '100%' }}>
      <Canvas
        orthographic
        camera={{ zoom: 1, position: [0, 0, 1], near: 0, far: 10 }}
        style={{ width: '100%', height: '100%', display: 'block' }}
        gl={{ antialias: false, alpha: false }}
        dpr={Math.min(window.devicePixelRatio, 1.5)}
      >
        <GradientMesh reducedMotion={prefersReducedMotion} />
      </Canvas>
    </div>
  );
};

export default MeshGradient;
