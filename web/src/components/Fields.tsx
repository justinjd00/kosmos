import { useCallback, useEffect, useRef, useState } from "react";
import { Sheet } from "../wasm/kosmos.js";
import { view } from "../lib/engine";

type Kind = "wave" | "heat" | "charge";

type Preset = {
  id: string;
  name: string;
  kind: Kind;
  blurb: string;
};

export const FIELDS: Preset[] = [
  {
    id: "ripples",
    name: "Ripples",
    kind: "wave",
    blurb:
      "One point source, driven forever. Circular crests spread outwards at a fixed speed and pass straight through each other where they meet.",
  },
  {
    id: "double-slit",
    name: "Double slit",
    kind: "wave",
    blurb:
      "Thomas Young, 1801. A wave meets a wall with two narrow gaps and lands behind it in bands rather than two stripes. Nothing that travels as a particle does this.",
  },
  {
    id: "single-slit",
    name: "Single slit",
    kind: "wave",
    blurb:
      "One gap, narrower than a wavelength. The wave leaves it as though the gap itself were the source — the same spreading, with no second slit to blame it on.",
  },
  {
    id: "lens",
    name: "Lens",
    kind: "wave",
    blurb:
      "A disc where the wave travels more slowly. Straight crests go in, curved crests come out, and the energy piles up at the focus. That is all a lens ever does.",
  },
  {
    id: "drum",
    name: "Drumhead",
    kind: "wave",
    blurb:
      "A closed box, struck once. The reflections come back, meet themselves, and settle into the standing patterns the boundary allows — the modes you hear as a pitch.",
  },
  {
    id: "harbour",
    name: "Harbour",
    kind: "wave",
    blurb:
      "A breakwater with one entrance. Waves squeeze through the mouth and spread inside instead of marching in — which is why harbour mouths are built narrow.",
  },
  {
    id: "hotspot",
    name: "Hot spot",
    kind: "heat",
    blurb:
      "One hot patch on a cold plate. Heat has no direction of its own; it only ever flows down the temperature gradient, and the peak can never rise again.",
  },
  {
    id: "radiator",
    name: "Radiator",
    kind: "heat",
    blurb:
      "A hot wall on the left, a cold one on the right, both held. Left alone the plate settles into a straight ramp of temperature between them and then stops changing.",
  },
  {
    id: "heatsink",
    name: "Insulation",
    kind: "heat",
    blurb:
      "A source hemmed in by two insulating blocks. Heat cannot cross them, so it goes around — the whole idea behind a heat sink's shape.",
  },
  {
    id: "dipole",
    name: "Dipole",
    kind: "charge",
    blurb:
      "One positive charge, one negative. The plane exactly between them sits at zero: every point on it is pulled equally hard in both directions.",
  },
  {
    id: "quadrupole",
    name: "Quadrupole",
    kind: "charge",
    blurb:
      "Two dipoles set back to back. Far away the four charges nearly cancel, so the potential dies off faster than any of them would alone.",
  },
  {
    id: "capacitor",
    name: "Capacitor",
    kind: "charge",
    blurb:
      "Two facing rows of opposite charge. Between the plates the field is almost uniform; outside them it almost vanishes. That confinement is what stores the energy.",
  },
];

const GRID_WIDTH = 480;
const GRID_HEIGHT = 270;

const GROUPS: { kind: Kind; title: string }[] = [
  { kind: "wave", title: "Waves" },
  { kind: "heat", title: "Heat" },
  { kind: "charge", title: "Charges" },
];

const HINTS: Record<Kind, string> = {
  wave: "Click to drop a pulse · Shift-drag to build a wall",
  heat: "Click to add heat · Shift-drag to build insulation",
  charge: "Drag a charge to move it",
};

type Stats = { time: number; energy: number; probe: number };

export default function Fields({
  presetId,
  onPreset,
  warm,
}: {
  presetId: string;
  onPreset: (id: string) => void;
  warm: number;
}) {
  const preset = FIELDS.find((entry) => entry.id === presetId) ?? FIELDS[0];

  const sheetRef = useRef<Sheet | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const wrapRef = useRef<HTMLDivElement | null>(null);
  const bufferRef = useRef<HTMLCanvasElement | null>(null);
  const frameRef = useRef<number | null>(null);
  const dragRef = useRef<{ mode: "poke" | "wall" | "source"; index: number } | null>(null);

  const [playing, setPlaying] = useState(true);
  const [auto, setAuto] = useState(true);
  const [gain, setGain] = useState(1.6);
  const [contours, setContours] = useState(true);
  const [speed, setSpeed] = useState(0.45);
  const [damping, setDamping] = useState(0);
  const [diffusivity, setDiffusivity] = useState(0.2);
  const [absorbing, setAbsorbing] = useState(true);
  const [stats, setStats] = useState<Stats>({ time: 0, energy: 0, probe: 0 });

  const build = useCallback(() => {
    sheetRef.current?.free();
    const sheet = new Sheet(preset.id, GRID_WIDTH, GRID_HEIGHT);
    sheetRef.current = sheet;

    setSpeed(sheet.speed());
    setDamping(sheet.damping());
    setDiffusivity(sheet.diffusivity());
    setAbsorbing(sheet.absorbing());

    let remaining = warm;
    while (remaining > 0) {
      const slice = Math.min(remaining, 0.2);
      sheet.advance(slice);
      remaining -= slice;
    }
  }, [preset.id, warm]);

  useEffect(() => {
    build();
    return () => {
      sheetRef.current?.free();
      sheetRef.current = null;
    };
  }, [build]);

  const paint = useCallback(() => {
    const sheet = sheetRef.current;
    const canvas = canvasRef.current;
    const wrap = wrapRef.current;
    if (!sheet || !canvas || !wrap) return;

    let buffer = bufferRef.current;
    if (!buffer) {
      buffer = document.createElement("canvas");
      buffer.width = GRID_WIDTH;
      buffer.height = GRID_HEIGHT;
      bufferRef.current = buffer;
    }

    const pointer = sheet.paint(auto ? 0 : gain, contours);
    const pixels = view(pointer, sheet.bytes());
    const source = buffer.getContext("2d");
    if (!source) return;
    source.putImageData(new ImageData(pixels, GRID_WIDTH, GRID_HEIGHT), 0, 0);

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

    const scale = Math.min(width / GRID_WIDTH, height / GRID_HEIGHT);
    const drawWidth = GRID_WIDTH * scale;
    const drawHeight = GRID_HEIGHT * scale;
    const left = (width - drawWidth) / 2;
    const top = (height - drawHeight) / 2;

    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = "high";
    ctx.drawImage(buffer, left, top, drawWidth, drawHeight);

    if (preset.kind === "charge") {
      const marks = sheet.sources();
      for (let i = 0; i < marks.length; i += 4) {
        const x = left + marks[i] * drawWidth;
        const y = top + marks[i + 1] * drawHeight;
        const positive = marks[i + 2] > 0;
        ctx.beginPath();
        ctx.arc(x, y, 9, 0, Math.PI * 2);
        ctx.fillStyle = positive ? "rgba(226, 96, 72, 0.9)" : "rgba(70, 150, 220, 0.9)";
        ctx.fill();
        ctx.strokeStyle = "rgba(240, 244, 252, 0.85)";
        ctx.lineWidth = 1.5;
        ctx.stroke();
        ctx.fillStyle = "#f4f7ff";
        ctx.font = "600 12px ui-monospace, monospace";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(positive ? "+" : "−", x, y + 0.5);
      }
    }

    ctx.strokeStyle = "rgba(120, 134, 158, 0.35)";
    ctx.lineWidth = 1;
    ctx.strokeRect(left + 0.5, top + 0.5, drawWidth - 1, drawHeight - 1);
  }, [auto, gain, contours, preset.kind]);

  useEffect(() => {
    let last = performance.now();

    const tick = (now: number) => {
      const sheet = sheetRef.current;
      if (sheet) {
        const delta = Math.min((now - last) / 1000, 0.05);
        last = now;
        if (playing && sheet.evolves()) {
          sheet.advance(delta);
        }
        paint();
        setStats({
          time: sheet.time(),
          energy: sheet.energy(),
          probe: sheet.probe(0.5, 0.5),
        });
      } else {
        last = now;
      }
      frameRef.current = requestAnimationFrame(tick);
    };

    frameRef.current = requestAnimationFrame(tick);
    return () => {
      if (frameRef.current !== null) cancelAnimationFrame(frameRef.current);
    };
  }, [playing, paint]);

  const locate = (event: React.PointerEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    const wrap = wrapRef.current;
    if (!canvas || !wrap) return null;

    const box = canvas.getBoundingClientRect();
    const scale = Math.min(box.width / GRID_WIDTH, box.height / GRID_HEIGHT);
    const drawWidth = GRID_WIDTH * scale;
    const drawHeight = GRID_HEIGHT * scale;
    const left = (box.width - drawWidth) / 2;
    const top = (box.height - drawHeight) / 2;

    const x = (event.clientX - box.left - left) / drawWidth;
    const y = (event.clientY - box.top - top) / drawHeight;
    if (x < 0 || x > 1 || y < 0 || y > 1) return null;
    return { x, y };
  };

  const nearestSource = (x: number, y: number) => {
    const sheet = sheetRef.current;
    if (!sheet) return -1;
    const marks = sheet.sources();
    let best = -1;
    let closest = 0.05;
    for (let i = 0; i < marks.length; i += 4) {
      const distance = Math.hypot(marks[i] - x, marks[i + 1] - y);
      if (distance < closest) {
        closest = distance;
        best = i / 4;
      }
    }
    return best;
  };

  const onDown = (event: React.PointerEvent<HTMLCanvasElement>) => {
    const sheet = sheetRef.current;
    const spot = locate(event);
    if (!sheet || !spot) return;
    event.currentTarget.setPointerCapture(event.pointerId);

    if (event.shiftKey) {
      dragRef.current = { mode: "wall", index: 0 };
      sheet.wall(spot.x, spot.y, 0.022, true);
      return;
    }

    const found = nearestSource(spot.x, spot.y);
    if (found >= 0) {
      dragRef.current = { mode: "source", index: found };
      return;
    }

    if (preset.kind === "charge") return;
    dragRef.current = { mode: "poke", index: 0 };
    sheet.poke(spot.x, spot.y, 0.035, preset.kind === "heat" ? 1.0 : 1.4);
  };

  const onMove = (event: React.PointerEvent<HTMLCanvasElement>) => {
    const sheet = sheetRef.current;
    const drag = dragRef.current;
    const spot = locate(event);
    if (!sheet || !drag || !spot) return;

    if (drag.mode === "wall") sheet.wall(spot.x, spot.y, 0.022, true);
    else if (drag.mode === "source") sheet.moveSource(drag.index, spot.x, spot.y);
    else if (preset.kind === "heat") sheet.poke(spot.x, spot.y, 0.03, 0.35);
  };

  const onUp = (event: React.PointerEvent<HTMLCanvasElement>) => {
    dragRef.current = null;
    event.currentTarget.releasePointerCapture(event.pointerId);
  };

  const reset = () => {
    build();
  };

  const applySpeed = (value: number) => {
    sheetRef.current?.setSpeed(value);
    setSpeed(value);
  };

  const applyDamping = (value: number) => {
    sheetRef.current?.setDamping(value);
    setDamping(value);
  };

  const applyDiffusivity = (value: number) => {
    sheetRef.current?.setDiffusivity(value);
    setDiffusivity(value);
  };

  const applyEdge = (value: boolean) => {
    sheetRef.current?.setEdge(value);
    setAbsorbing(value);
  };

  const clearWalls = () => {
    sheetRef.current?.clearWalls();
  };

  return (
    <div className="body">
      <aside className="panel">
        <div className="panel-head">
          <h2>Fields</h2>
        </div>

        {GROUPS.map((group) => (
          <div className="group" key={group.kind}>
            <h3>{group.title}</h3>
            <div className="system-list">
              {FIELDS.filter((entry) => entry.kind === group.kind).map((entry) => (
                <button
                  key={entry.id}
                  className={entry.id === preset.id ? "system on" : "system"}
                  onClick={() => onPreset(entry.id)}
                >
                  {entry.name}
                </button>
              ))}
            </div>
          </div>
        ))}

        <p className="blurb">{preset.blurb}</p>

        {preset.kind === "wave" && (
          <div className="section">
            <h3>Medium</h3>
            <label className="slider">
              <span className="slider-name">c</span>
              <input
                type="range"
                min={0.12}
                max={0.5}
                step={0.005}
                value={speed}
                onChange={(event) => applySpeed(Number(event.target.value))}
              />
              <span className="slider-value">{speed.toFixed(3)}</span>
            </label>
            <label className="slider">
              <span className="slider-name">γ</span>
              <input
                type="range"
                min={0}
                max={0.004}
                step={0.00002}
                value={damping}
                onChange={(event) => applyDamping(Number(event.target.value))}
              />
              <span className="slider-value">{damping.toFixed(5)}</span>
            </label>
            <div className="entry-tools">
              <button className={absorbing ? "chip on" : "chip"} onClick={() => applyEdge(true)}>
                open edges
              </button>
              <button className={absorbing ? "chip" : "chip on"} onClick={() => applyEdge(false)}>
                hard walls
              </button>
            </div>
          </div>
        )}

        {preset.kind === "heat" && (
          <div className="section">
            <h3>Medium</h3>
            <label className="slider">
              <span className="slider-name">α</span>
              <input
                type="range"
                min={0.02}
                max={0.24}
                step={0.002}
                value={diffusivity}
                onChange={(event) => applyDiffusivity(Number(event.target.value))}
              />
              <span className="slider-value">{diffusivity.toFixed(3)}</span>
            </label>
          </div>
        )}

        <div className="section">
          <h3>View</h3>
          <div className="entry-tools">
            <button className={auto ? "chip on" : "chip"} onClick={() => setAuto((v) => !v)}>
              auto contrast
            </button>
            <button className={contours ? "chip on" : "chip"} onClick={() => setContours((v) => !v)}>
              contours
            </button>
          </div>
          {!auto && (
            <label className="slider">
              <span className="slider-name">◐</span>
              <input
                type="range"
                min={0.2}
                max={8}
                step={0.05}
                value={gain}
                onChange={(event) => setGain(Number(event.target.value))}
              />
              <span className="slider-value">{gain.toFixed(2)}</span>
            </label>
          )}
        </div>

        <div className="section">
          <div className="entry-tools">
            {preset.kind !== "charge" && (
              <button className="chip on" onClick={() => setPlaying((p) => !p)}>
                {playing ? "Pause" : "Play"}
              </button>
            )}
            <button className="chip" onClick={reset}>
              Reset
            </button>
            <button className="chip" onClick={clearWalls}>
              Clear walls
            </button>
          </div>
        </div>

        <div className="section">
          <h3>Readout</h3>
          {preset.kind !== "charge" && (
            <>
              <div className="stat">
                <span>time</span>
                <span>{stats.time.toFixed(2)}</span>
              </div>
              <div className="stat">
                <span>{preset.kind === "wave" ? "energy" : "mean heat"}</span>
                <span>{stats.energy.toExponential(3)}</span>
              </div>
            </>
          )}
          <div className="stat">
            <span>centre</span>
            <span>{stats.probe.toFixed(4)}</span>
          </div>
        </div>
      </aside>

      <main className="stage">
        <div className="plot" ref={wrapRef}>
          <canvas
            ref={canvasRef}
            className="grabbable"
            onPointerDown={onDown}
            onPointerMove={onMove}
            onPointerUp={onUp}
            onPointerCancel={onUp}
          />
          <div className="corner-hint">{HINTS[preset.kind]}</div>
        </div>
      </main>
    </div>
  );
}
