import { useCallback, useEffect, useRef, useState } from "react";
import Plot, { type Track, type View } from "./Plot";
import { compile, PALETTE, PARAM_NAMES, type Compiled, type SyntaxError } from "../lib/engine";

type Entry = {
  id: string;
  source: string;
  color: string;
  visible: boolean;
  derivative: boolean;
  markers: boolean;
  error: SyntaxError | null;
};

const DEFAULT_VIEW: View = { xMin: -10, xMax: 10, yMin: -6.5, yMax: 6.5 };
const SEEDS = ["sin(x)", "0.4x^2 - 3"];

let counter = 0;

function blank(source: string): Entry {
  counter += 1;
  return {
    id: `f${counter}`,
    source,
    color: PALETTE[(counter - 1) % PALETTE.length],
    visible: true,
    derivative: false,
    markers: false,
    error: null,
  };
}

export default function Functions() {
  const [entries, setEntries] = useState<Entry[]>(() => SEEDS.map(blank));
  const [view, setView] = useState<View>(DEFAULT_VIEW);
  const [params, setParams] = useState<[number, number, number, number]>([1, 1, 1, 1]);
  const [time, setTime] = useState(0);
  const [playing, setPlaying] = useState(false);
  const frameRef = useRef<number | null>(null);

  const handles = useRef(new Map<string, Compiled>());

  const applySource = useCallback((id: string, source: string): SyntaxError | null => {
    handles.current.get(id)?.handle.free();
    handles.current.delete(id);

    const result = compile(source);
    if (result.ok) {
      handles.current.set(id, result.value);
      return null;
    }
    return result.error.message ? result.error : null;
  }, []);

  useEffect(() => {
    const compiled = entries.map((entry) => ({
      id: entry.id,
      error: applySource(entry.id, entry.source),
    }));
    setEntries((current) =>
      current.map((entry) => {
        const found = compiled.find((item) => item.id === entry.id);
        return found ? { ...entry, error: found.error } : entry;
      }),
    );

    const map = handles.current;
    return () => {
      for (const compiledEntry of map.values()) compiledEntry.handle.free();
      map.clear();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    for (const compiled of handles.current.values()) {
      params.forEach((value, index) => compiled.handle.setParam(index, value));
    }
  }, [entries, params]);

  useEffect(() => {
    if (!playing) return;
    let last = performance.now();
    const tick = (now: number) => {
      const delta = (now - last) / 1000;
      last = now;
      setTime((t) => t + delta);
      frameRef.current = requestAnimationFrame(tick);
    };
    frameRef.current = requestAnimationFrame(tick);
    return () => {
      if (frameRef.current !== null) cancelAnimationFrame(frameRef.current);
    };
  }, [playing]);

  const setSource = (id: string, source: string) => {
    const error = applySource(id, source);
    setEntries((current) =>
      current.map((entry) => (entry.id === id ? { ...entry, source, error } : entry)),
    );
  };

  const toggle = (id: string, key: "visible" | "derivative" | "markers") => {
    setEntries((current) =>
      current.map((entry) => (entry.id === id ? { ...entry, [key]: !entry[key] } : entry)),
    );
  };

  const remove = (id: string) => {
    handles.current.get(id)?.handle.free();
    handles.current.delete(id);
    setEntries((current) => current.filter((entry) => entry.id !== id));
  };

  const add = () => setEntries((current) => [...current, blank("")]);

  const tracks: Track[] = entries
    .filter((entry) => entry.visible && handles.current.has(entry.id))
    .map((entry) => ({
      id: entry.id,
      label: entry.source,
      color: entry.color,
      compiled: handles.current.get(entry.id)!,
      showDerivative: entry.derivative,
      showMarkers: entry.markers,
    }));

  const usedParams = PARAM_NAMES.map((_, index) =>
    entries.some((entry) => handles.current.get(entry.id)?.usesParam[index]),
  );
  const usesTime = entries.some((entry) => handles.current.get(entry.id)?.usesTime);

  return (
    <div className="body">
      <aside className="panel">
        <div className="panel-head">
          <h2>Functions</h2>
          <button className="ghost" onClick={add} title="Add a function">
            +
          </button>
        </div>

        <div className="entries">
          {entries.map((entry) => {
            const compiled = handles.current.get(entry.id);
            return (
              <div className="entry" key={entry.id}>
                <div className="entry-top">
                  <button
                    className="swatch"
                    style={{
                      background: entry.visible ? entry.color : "transparent",
                      borderColor: entry.color,
                    }}
                    onClick={() => toggle(entry.id, "visible")}
                    title="Show or hide"
                  />
                  <input
                    className={entry.error ? "formula invalid" : "formula"}
                    value={entry.source}
                    spellCheck={false}
                    placeholder="e.g. a*sin(x - t)"
                    onChange={(event) => setSource(entry.id, event.target.value)}
                  />
                  <button className="ghost" onClick={() => remove(entry.id)} title="Remove">
                    ×
                  </button>
                </div>

                {entry.error && (
                  <div className="error">
                    <code>
                      {entry.source.slice(0, entry.error.at)}
                      <mark>{entry.source.slice(entry.error.at, entry.error.at + 1) || "·"}</mark>
                      {entry.source.slice(entry.error.at + 1)}
                    </code>
                    <span>{entry.error.message}</span>
                  </div>
                )}

                {compiled && (
                  <div className="entry-tools">
                    <button
                      className={entry.derivative ? "chip on" : "chip"}
                      onClick={() => toggle(entry.id, "derivative")}
                    >
                      derivative
                    </button>
                    <button
                      className={entry.markers ? "chip on" : "chip"}
                      onClick={() => toggle(entry.id, "markers")}
                    >
                      roots &amp; extrema
                    </button>
                  </div>
                )}

                {compiled && entry.derivative && (
                  <div className="derivative">f′(x) = {compiled.handle.derivativeText()}</div>
                )}
              </div>
            );
          })}
        </div>

        {usedParams.some(Boolean) && (
          <div className="section">
            <h3>Parameters</h3>
            {PARAM_NAMES.map((name, index) =>
              usedParams[index] ? (
                <label className="slider" key={name}>
                  <span className="slider-name">{name}</span>
                  <input
                    type="range"
                    min={-5}
                    max={5}
                    step={0.01}
                    value={params[index]}
                    onChange={(event) => {
                      const next = [...params] as [number, number, number, number];
                      next[index] = Number(event.target.value);
                      setParams(next);
                    }}
                  />
                  <span className="slider-value">{params[index].toFixed(2)}</span>
                </label>
              ) : null,
            )}
          </div>
        )}

        {usesTime && (
          <div className="section">
            <h3>Time</h3>
            <div className="time-row">
              <button className="chip on" onClick={() => setPlaying((p) => !p)}>
                {playing ? "Pause" : "Play"}
              </button>
              <input
                type="range"
                min={0}
                max={20}
                step={0.01}
                value={time % 20}
                onChange={(event) => setTime(Number(event.target.value))}
              />
              <span className="slider-value">{time.toFixed(2)}</span>
            </div>
          </div>
        )}

        <div className="section legend">
          <h3>Syntax</h3>
          <p>
            <code>2x</code>, <code>3sin(x)</code> and <code>(x+1)(x-1)</code> work without a
            multiplication sign. Available: <code>sin cos tan asin acos atan sinh cosh tanh exp ln
            log2 log10 sqrt cbrt abs sign floor ceil round</code>, plus{" "}
            <code>min max atan2 hypot pow log</code> taking two arguments. Constants:{" "}
            <code>pi tau e phi</code>. Variables: <code>x</code>, time <code>t</code>, and the
            parameters <code>a b c d</code>.
          </p>
        </div>
      </aside>

      <main className="stage">
        <Plot tracks={tracks} view={view} onView={setView} time={time} />
      </main>
    </div>
  );
}
