import { Canvas, useThree } from '@react-three/fiber';
import { useEffect, useLayoutEffect, useMemo, useRef } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { WebGPURenderer } from 'three/webgpu';

import { brickLength } from '../geometry';
import type { Plan, Placement, WallSpec } from '../types';

// Materials cannot read CSS variables, so the palette is mirrored here.
const COLORS = {
  bg: '#fffdee',
  ground: '#f0ecd4',
  brick: '#c1b7a4',
  cut: '#d6cdba',
  accent: '#f74823',
  ink: '#1a024d',
  wine: '#721e3c',
};

// The planner speaks millimeters; the scene is in meters. Plan x maps to
// three.js x, plan y to z, and height to y.
const MM = 0.001;

interface WallSceneProps {
  plan: Plan;
  /** How many bricks are on the walls at the current playback position. */
  placedCount: number;
}

export default function WallScene({ plan, placedCount }: WallSceneProps) {
  const { spec } = plan;
  const maxDim = Math.max(spec.width, spec.length, spec.height) * MM;
  return (
    <div className="scene-holder">
      {/* WebGPU when the browser has it, WebGL otherwise (the renderer
          falls back by itself). frameloop="demand" renders only when the
          camera moves or the plan/playback changes — idle cost is zero. */}
      <Canvas
        frameloop="demand"
        gl={async (props) => {
          const renderer = new WebGPURenderer({
            ...(props as ConstructorParameters<typeof WebGPURenderer>[0]),
            antialias: true,
          });
          await renderer.init();
          return renderer;
        }}
        // Camera starts on the -z side so the front wall (and its opening)
        // faces the viewer.
        camera={{ fov: 40, position: [maxDim * 1.4, maxDim * 1.05, -maxDim * 1.8] }}
      >
        <color attach="background" args={[COLORS.bg]} />
        <ambientLight intensity={1.6} />
        <directionalLight position={[4, 8, -6]} intensity={1.8} />
        <directionalLight position={[-6, 3, 4]} intensity={0.7} />
        <Controls targetY={(spec.height * MM) / 2} />
        <Bricks plan={plan} placedCount={placedCount} />
        <Ghost spec={spec} />
        {/* The floor is exactly the building's footprint. */}
        <mesh rotation-x={-Math.PI / 2} position-y={-0.002}>
          <planeGeometry args={[spec.width * MM, spec.length * MM]} />
          <meshBasicMaterial color={COLORS.ground} />
        </mesh>
      </Canvas>
    </div>
  );
}

/// Hand-rolled orbit controls: three's own OrbitControls wired into the
/// fiber camera, kept dependency-free (no drei).
function Controls({ targetY }: { targetY: number }) {
  const { camera, gl, invalidate } = useThree();
  const controlsRef = useRef<OrbitControls | null>(null);
  useEffect(() => {
    const controls = new OrbitControls(camera, gl.domElement);
    // In demand mode, camera movement must ask for a frame explicitly.
    controls.addEventListener('change', () => invalidate());
    controlsRef.current = controls;
    return () => controls.dispose();
  }, [camera, gl, invalidate]);
  useEffect(() => {
    const controls = controlsRef.current;
    if (controls) {
      controls.target.set(0, targetY, 0);
      controls.update();
    }
  }, [targetY]);
  return null;
}

/// Where the build is heading: a wireframe of the target envelope, and
/// the opening marked in the front wall before any brick reaches it.
function Ghost({ spec }: { spec: WallSpec }) {
  const envelope = useMemo(
    () =>
      new THREE.EdgesGeometry(
        new THREE.BoxGeometry(spec.width * MM, spec.height * MM, spec.length * MM),
      ),
    [spec.width, spec.height, spec.length],
  );
  const opening = useMemo(
    () =>
      spec.opening
        ? new THREE.EdgesGeometry(
            new THREE.BoxGeometry(
              spec.opening.width * MM,
              spec.opening.height * MM,
              spec.brick.width * MM,
            ),
          )
        : null,
    [spec.opening, spec.brick.width],
  );
  const op = spec.opening;
  return (
    <group>
      <lineSegments geometry={envelope} position={[0, (spec.height * MM) / 2, 0]}>
        <lineBasicMaterial color={COLORS.ink} transparent opacity={0.3} />
      </lineSegments>
      {op && opening && (
        <group
          position={[
            (op.x + op.width / 2) * MM - (spec.width * MM) / 2,
            (op.sill_height + op.height / 2) * MM,
            (spec.brick.width * MM) / 2 - (spec.length * MM) / 2,
          ]}
        >
          <lineSegments geometry={opening}>
            <lineBasicMaterial color={COLORS.wine} transparent opacity={0.55} />
          </lineSegments>
          <mesh>
            <boxGeometry
              args={[op.width * MM, op.height * MM, (spec.brick.width * MM) * 0.98]}
            />
            <meshBasicMaterial color={COLORS.wine} transparent opacity={0.07} depthWrite={false} />
          </mesh>
        </group>
      )}
    </group>
  );
}

function baseColor(p: Placement): string {
  return p.kind.type === 'Cut' ? COLORS.cut : COLORS.brick;
}

/// All bricks live in one InstancedMesh: matrices are written once per
/// plan, and playback only moves the instance count — so scrubbing a
/// multi-thousand-step plan costs nothing.
function Bricks({ plan, placedCount }: WallSceneProps) {
  const meshRef = useRef<THREE.InstancedMesh>(null);
  const invalidate = useThree((state) => state.invalidate);
  const { spec } = plan;

  // Replay order comes from the steps, not the placements array: the
  // plan is the contract, and the simulator only executes it.
  const placeOrder = useMemo(() => {
    const byId = new Map(plan.placements.map((p) => [p.id, p]));
    return plan.steps.flatMap((s) =>
      s.action.type === 'PlaceBrick' ? [byId.get(s.action.placement_id)!] : [],
    );
  }, [plan]);

  useLayoutEffect(() => {
    const mesh = meshRef.current;
    if (!mesh) return;
    const matrix = new THREE.Matrix4();
    const rotation = new THREE.Quaternion();
    const position = new THREE.Vector3();
    const scale = new THREE.Vector3();
    const color = new THREE.Color();
    const t = spec.brick.width;
    placeOrder.forEach((p, i) => {
      const alongX = p.wall === 'South' || p.wall === 'North';
      const len = brickLength(p.kind, spec);
      const sizeX = (alongX ? len : t) * MM;
      const sizeY = spec.brick.height * MM;
      const sizeZ = (alongX ? t : len) * MM;
      position.set(
        p.x * MM + sizeX / 2 - (spec.width * MM) / 2,
        p.z * MM + sizeY / 2,
        p.y * MM + sizeZ / 2 - (spec.length * MM) / 2,
      );
      scale.set(sizeX, sizeY, sizeZ);
      matrix.compose(position, rotation, scale);
      mesh.setMatrixAt(i, matrix);
      mesh.setColorAt(i, color.set(baseColor(p)));
    });
    mesh.instanceMatrix.needsUpdate = true;
    if (mesh.instanceColor) mesh.instanceColor.needsUpdate = true;
    invalidate();
  }, [placeOrder, spec, invalidate]);

  // Playback: draw the first placedCount instances, most recent in accent.
  const prevLast = useRef(-1);
  useLayoutEffect(() => {
    const mesh = meshRef.current;
    if (!mesh) return;
    const color = new THREE.Color();
    if (prevLast.current >= 0 && prevLast.current < placeOrder.length) {
      const p = placeOrder[prevLast.current];
      mesh.setColorAt(prevLast.current, color.set(baseColor(p)));
    }
    const last = placedCount - 1;
    if (last >= 0) {
      mesh.setColorAt(last, color.set(COLORS.accent));
    }
    prevLast.current = last;
    mesh.count = placedCount;
    if (mesh.instanceColor) mesh.instanceColor.needsUpdate = true;
    invalidate();
  }, [placedCount, placeOrder, invalidate]);

  return (
    <instancedMesh
      ref={meshRef}
      args={[undefined, undefined, Math.max(placeOrder.length, 1)]}
      frustumCulled={false}
    >
      <boxGeometry />
      <meshLambertMaterial />
    </instancedMesh>
  );
}
