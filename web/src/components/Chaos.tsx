import { useCallback, useEffect, useRef, useState } from "react";
import { System } from "../wasm/kosmos.js";

type Preset = {
  id: string;
  name: string;
  blurb: string;
  params: string[];
  ranges: [number, number][];
};

export const PRESETS: Preset[] = [
  {
    id: "lorenz",
    name: "Lorenz",
    blurb:
      "Edward Lorenz, 1963. A stripped-down model of convection in the atmosphere. Rounding a number from six digits to three changed his weather forecast completely — which is how chaos was discovered.",
    params: ["σ", "ρ", "β"],
    ranges: [
      [1, 20],
      [0.5, 45],
      [0.5, 6],
    ],
  },
  {
    id: "rossler",
    name: "Rössler",
    blurb:
      "Otto Rössler designed this in 1976 to be the simplest system that could still be chaotic. It has a single quadratic term. Raise c and watch the band split once, then twice, then dissolve.",
    params: ["a", "b", "c"],
    ranges: [
      [0.05, 0.5],
      [0.05, 2],
      [2, 18],
    ],
  },
  {
    id: "aizawa",
    name: "Aizawa",
    blurb: "A sphere with an axis drilled through it. Beautiful rather than famous.",
    params: ["a", "b", "c", "d"],
    ranges: [
      [0.3, 1.4],
      [0.1, 1.2],
      [0.2, 1.2],
      [1, 6],
    ],
  },
  {
    id: "thomas",
    name: "Thomas",
    blurb:
      "Cyclically symmetric: each coordinate is driven by the sine of the next. Lower b and the orbit fills more of the lattice.",
    params: ["b"],
    ranges: [[0.05, 0.33]],
  },
  {
    id: "halvorsen",
    name: "Halvorsen",
    blurb: "Three-fold symmetry, three curled arms.",
    params: ["a"],
    ranges: [[0.8, 2.2]],
  },
  {
    id: "chen",
    name: "Chen",
    blurb: "A cousin of Lorenz that is not topologically equivalent to it.",
    params: ["a", "b", "c"],
    ranges: [
      [2, 8],
      [-14, -4],
      [-1.5, 0.5],
    ],
  },
  {
    id: "double-pendulum",
    name: "Double pendulum",
    blurb:
      "Two rods, four numbers, no randomness — and no way to predict where it will be in a minute. The energy readout stays flat, which is how you know the integrator is honest.",
    params: ["m₁", "m₂", "l₁", "l₂", "g"],
    ranges: [
      [0.2, 3],
      [0.2, 3],
      [0.3, 2],
      [0.3, 2],
      [1, 20],
    ],
  },
  {
    id: "three-body",
    name: "Three-body problem",
    blurb:
      "Three equal masses on the figure-eight orbit found by Cristopher Moore in 1993. It is exactly periodic — until you nudge it. Then it is not.",
    params: ["m₁", "m₂", "m₃"],
    ranges: [
      [0.5, 1.6],
      [0.5, 1.6],
      [0.5, 1.6],
    ],
  },
];

const TWIN_COLOR = "#fb7185";
const MAIN_COLOR = "#5eead4";
const BODY_COLORS = ["#5eead4", "#a78bfa", "#fbbf24"];

type Props = {
  presetId: string;
  onPreset: (id: string) => void;
  twin: boolean;
  onTwin: (value: boolean) => void;
  warm?: number;
};

export default function Chaos({ presetId, onPreset, twin, onTwin, warm = 0 }: Props) {
  const preset = PRESETS.find((p) => p.id === presetId) ?? PRESETS[0];

  const wrapRef = useRef<HTMLDivElement | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const mainRef = useRef<System | null>(null);
  const twinRef = useRef<System | null>(null);
  const frameRef = useRef<number | null>(null);
  const angleRef = useRef({ yaw: 0.28, pitch: 0.12 });
  const dragRef = useRef<{ x: number; y: number; yaw: number; pitch: number } | null>(null);
  const viewRef = useRef({ cx: 0, cy: 0, scale: 1 });
  const tickRef = useRef(0);

  const [params, setParams] = useState<number[]>([]);
  const [running, setRunning] = useState(true);
  const [trailLength, setTrailLength] = useState(12000);
  const [speed, setSpeed] = useState(1);
  const [generation, setGeneration] = useState(0);
  const [stats, setStats] = useState({ time: 0, energy: NaN, separation: NaN });

  const build = useCallback(() => {
    mainRef.current?.free();
    const main = new System(preset.id);
    const count = main.paramCount();
    const values = Array.from({ length: count }, (_, i) => defaultParam(preset, i));
    values.forEach((value, index) => main.setParam(index, value));
    main.advance(8);
    main.clearTrail();
    main.advance(10);

    mainRef.current = main;
    viewRef.current = { cx: 0, cy: 0, scale: 0 };
    tickRef.current = 0;
    setParams(values);
    setGeneration((value) => value + 1);
  }, [preset]);

  useEffect(() => {
    build();
    return () => {
      mainRef.current?.free();
      mainRef.current = null;
    };
  }, [build]);

  useEffect(() => {
    const main = mainRef.current;
    if (!main) return;
    params.forEach((value, index) => {
      main.setParam(index, value);
      twinRef.current?.setParam(index, value);
    });
  }, [params]);

  useEffect(() => {
    if (!twin) {
      twinRef.current?.free();
      twinRef.current = null;
      return;
    }
    const main = mainRef.current;
    if (!main) return;

    const clone = new System(preset.id);
    params.forEach((value, index) => clone.setParam(index, value));
    clone.setState(main.state());
    clone.setTime(main.time());
    clone.nudge(0, 1e-7);
    twinRef.current = clone;
    return () => {
      if (twinRef.current === clone) twinRef.current = null;
      clone.free();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [twin, preset.id, generation]);

  useEffect(() => {
    if (warm <= 0) return;
    const main = mainRef.current;
    if (!main) return;

    let remaining = warm;
    while (remaining > 0) {
      const slice = Math.min(remaining, 10);
      main.advance(slice);
      twinRef.current?.advance(slice);
      remaining -= slice;
    }

    const a = main.state();
    const b = twinRef.current?.state();
    setStats({
      time: main.time(),
      energy: main.energy(),
      separation: b ? Math.hypot(...a.map((value, index) => value - b[index])) : NaN,
    });
  }, [warm, twin, generation]);

  const render = useCallback(() => {
    const canvas = canvasRef.current;
    const wrap = wrapRef.current;
    const main = mainRef.current;
    if (!canvas || !wrap || !main) return;

    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const width = wrap.clientWidth;
    const height = wrap.clientHeight;
    if (width === 0 || height === 0) return;

    if (canvas.width !== Math.round(width * dpr)) {
      canvas.width = Math.round(width * dpr);
      canvas.height = Math.round(height * dpr);
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
    }

    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.fillStyle = "#08090d";
    ctx.fillRect(0, 0, width, height);

    const { yaw, pitch } = angleRef.current;

    tickRef.current += 1;
    if (tickRef.current % 12 === 1 && main.trailLength() > 200) {
      const bounds = main.bounds(yaw, pitch);
      const spanX = Math.max(bounds[1] - bounds[0], 1e-6);
      const spanY = Math.max(bounds[3] - bounds[2], 1e-6);
      const target = Math.min((width * 0.82) / spanX, (height * 0.82) / spanY);
      const cx = (bounds[0] + bounds[1]) / 2;
      const cy = (bounds[2] + bounds[3]) / 2;
      const view = viewRef.current;
      if (view.scale === 0) {
        viewRef.current = { cx, cy, scale: target };
      } else {
        viewRef.current = {
          cx: view.cx + (cx - view.cx) * 0.08,
          cy: view.cy + (cy - view.cy) * 0.08,
          scale: view.scale + (target - view.scale) * 0.08,
        };
      }
    }

    const { cx, cy, scale } = viewRef.current;
    const toX = (x: number) => width / 2 + (x - cx) * scale;
    const toY = (y: number) => height / 2 - (y - cy) * scale;

    const drawTrail = (system: System, color: string) => {
      const points = system.trail(yaw, pitch, trailLength);
      const total = points.length / 2;
      if (total < 2) return;

      const segments = 20;
      const per = Math.ceil(total / segments);
      ctx.lineJoin = "round";
      ctx.lineCap = "round";

      for (let s = 0; s < segments; s += 1) {
        const from = s * per;
        const to = Math.min((s + 1) * per + 1, total);
        if (to - from < 2) continue;
        const age = s / (segments - 1);
        ctx.globalAlpha = 0.06 + age * 0.94;
        ctx.lineWidth = 0.5 + age * 1.3;
        ctx.strokeStyle = color;
        ctx.beginPath();
        for (let i = from; i < to; i += 1) {
          const px = toX(points[i * 2]);
          const py = toY(points[i * 2 + 1]);
          if (i === from) ctx.moveTo(px, py);
          else ctx.lineTo(px, py);
        }
        ctx.stroke();
      }
      ctx.globalAlpha = 1;
    };

    if (preset.id === "double-pendulum") {
      const drawPendulum = (system: System, color: string, alpha: number) => {
        const p = system.positions();
        ctx.globalAlpha = alpha;
        ctx.strokeStyle = "#3b4457";
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(toX(0), toY(0));
        ctx.lineTo(toX(p[0]), toY(p[1]));
        ctx.lineTo(toX(p[2]), toY(p[3]));
        ctx.stroke();

        ctx.fillStyle = color;
        for (const [x, y] of [
          [p[0], p[1]],
          [p[2], p[3]],
        ]) {
          ctx.beginPath();
          ctx.arc(toX(x), toY(y), 6, 0, Math.PI * 2);
          ctx.fill();
        }
        ctx.globalAlpha = 1;
      };

      drawTrail(main, MAIN_COLOR);
      if (twinRef.current) drawTrail(twinRef.current, TWIN_COLOR);
      drawPendulum(main, MAIN_COLOR, 1);
      if (twinRef.current) drawPendulum(twinRef.current, TWIN_COLOR, 0.8);
    } else if (preset.id === "three-body") {
      drawTrail(main, MAIN_COLOR);
      if (twinRef.current) drawTrail(twinRef.current, TWIN_COLOR);
      const p = main.positions();
      for (let body = 0; body < 3; body += 1) {
        ctx.fillStyle = BODY_COLORS[body];
        ctx.shadowColor = BODY_COLORS[body];
        ctx.shadowBlur = 12;
        ctx.beginPath();
        ctx.arc(toX(p[body * 2]), toY(p[body * 2 + 1]), 7, 0, Math.PI * 2);
        ctx.fill();
        ctx.shadowBlur = 0;
      }
    } else {
      ctx.shadowColor = MAIN_COLOR;
      ctx.shadowBlur = 6;
      drawTrail(main, MAIN_COLOR);
      ctx.shadowBlur = 0;
      if (twinRef.current) drawTrail(twinRef.current, TWIN_COLOR);
    }
  }, [preset.id, trailLength]);

  useEffect(() => {
    let last = performance.now();

    const loop = (now: number) => {
      const delta = Math.min((now - last) / 1000, 0.05);
      last = now;

      const main = mainRef.current;
      if (main && running) {
        main.advance(delta * speed);
        twinRef.current?.advance(delta * speed);

        const a = main.state();
        const b = twinRef.current?.state();
        let separation = NaN;
        if (b) {
          separation = Math.hypot(...a.map((value, index) => value - b[index]));
        }
        setStats({ time: main.time(), energy: main.energy(), separation });
      }

      render();
      frameRef.current = requestAnimationFrame(loop);
    };

    frameRef.current = requestAnimationFrame(loop);
    return () => {
      if (frameRef.current !== null) cancelAnimationFrame(frameRef.current);
    };
  }, [running, speed, render]);

  const onPointerDown = (event: React.PointerEvent<HTMLCanvasElement>) => {
    if (!mainRef.current?.isSpatial()) return;
    (event.target as HTMLCanvasElement).setPointerCapture(event.pointerId);
    dragRef.current = {
      x: event.clientX,
      y: event.clientY,
      yaw: angleRef.current.yaw,
      pitch: angleRef.current.pitch,
    };
  };

  const onPointerMove = (event: React.PointerEvent<HTMLCanvasElement>) => {
    const drag = dragRef.current;
    if (!drag) return;
    angleRef.current = {
      yaw: drag.yaw + (event.clientX - drag.x) * 0.008,
      pitch: Math.max(
        -Math.PI / 2,
        Math.min(Math.PI / 2, drag.pitch + (event.clientY - drag.y) * 0.008),
      ),
    };
  };

  const endDrag = (event: React.PointerEvent<HTMLCanvasElement>) => {
    (event.target as HTMLCanvasElement).releasePointerCapture(event.pointerId);
    dragRef.current = null;
  };

  const spatial = preset.id !== "double-pendulum" && preset.id !== "three-body";

  return (
    <div className="body">
      <aside className="panel">
        <div className="panel-head">
          <h2>Systems</h2>
        </div>

        <div className="system-list">
          {PRESETS.map((item) => (
            <button
              key={item.id}
              className={item.id === preset.id ? "system on" : "system"}
              onClick={() => onPreset(item.id)}
            >
              {item.name}
            </button>
          ))}
        </div>

        <p className="blurb">{preset.blurb}</p>

        <div className="section">
          <div className="entry-tools">
            <button className={running ? "chip on" : "chip"} onClick={() => setRunning((r) => !r)}>
              {running ? "Pause" : "Play"}
            </button>
            <button className="chip" onClick={build}>
              Reset
            </button>
            <button className={twin ? "chip on" : "chip"} onClick={() => onTwin(!twin)}>
              Butterfly twin
            </button>
          </div>
          {twin && (
            <p className="blurb small">
              A second copy started one ten-millionth away from the first. Same equations, same
              machine, no randomness anywhere.
            </p>
          )}
        </div>

        <div className="section">
          <h3>Parameters</h3>
          {params.map((value, index) => (
            <label className="slider" key={index}>
              <span className="slider-name">{preset.params[index] ?? `p${index}`}</span>
              <input
                type="range"
                min={preset.ranges[index]?.[0] ?? 0}
                max={preset.ranges[index]?.[1] ?? 1}
                step={0.001}
                value={value}
                onChange={(event) => {
                  const next = [...params];
                  next[index] = Number(event.target.value);
                  setParams(next);
                }}
              />
              <span className="slider-value">{value.toFixed(3)}</span>
            </label>
          ))}

          <label className="slider">
            <span className="slider-name">↻</span>
            <input
              type="range"
              min={0.1}
              max={4}
              step={0.05}
              value={speed}
              onChange={(event) => setSpeed(Number(event.target.value))}
            />
            <span className="slider-value">{speed.toFixed(2)}×</span>
          </label>

          <label className="slider">
            <span className="slider-name">~</span>
            <input
              type="range"
              min={500}
              max={40000}
              step={500}
              value={trailLength}
              onChange={(event) => setTrailLength(Number(event.target.value))}
            />
            <span className="slider-value">{(trailLength / 1000).toFixed(0)}k</span>
          </label>
        </div>

        <div className="section">
          <h3>Readout</h3>
          <div className="stat">
            <span>time</span>
            <span>{stats.time.toFixed(2)}</span>
          </div>
          {Number.isFinite(stats.energy) && (
            <div className="stat">
              <span>energy</span>
              <span>{stats.energy.toFixed(6)}</span>
            </div>
          )}
          {Number.isFinite(stats.separation) && (
            <div className="stat">
              <span>separation</span>
              <span>{stats.separation.toExponential(3)}</span>
            </div>
          )}
        </div>
      </aside>

      <main className="stage">
        <div className="plot" ref={wrapRef}>
          <canvas
            ref={canvasRef}
            className={spatial ? "grabbable" : ""}
            onPointerDown={onPointerDown}
            onPointerMove={onPointerMove}
            onPointerUp={endDrag}
            onPointerCancel={endDrag}
          />
          {spatial && <div className="corner-hint">Drag to rotate</div>}
        </div>
      </main>
    </div>
  );
}

function defaultParam(preset: Preset, index: number): number {
  const defaults: Record<string, number[]> = {
    lorenz: [10, 28, 8 / 3],
    rossler: [0.2, 0.2, 5.7],
    aizawa: [0.95, 0.7, 0.6, 3.5],
    thomas: [0.19],
    halvorsen: [1.4],
    chen: [5, -10, -0.38],
    "double-pendulum": [1, 1, 1, 1, 9.81],
    "three-body": [1, 1, 1],
  };
  return defaults[preset.id]?.[index] ?? 1;
}
