import init, {
  Function as WasmFunction,
  niceStep,
  type InitOutput,
} from "../wasm/kosmos.js";

let ready: Promise<void> | null = null;
let engine: InitOutput | null = null;

export function boot(): Promise<void> {
  if (!ready) {
    ready = init().then((output) => {
      engine = output;
    });
  }
  return ready;
}

export function view(pointer: number, length: number): Uint8ClampedArray<ArrayBuffer> {
  if (!engine) {
    throw new Error("the engine has not started yet");
  }
  return new Uint8ClampedArray(engine.memory.buffer as ArrayBuffer, pointer, length);
}

export type SyntaxError = { message: string; at: number };

export type Compiled = {
  handle: WasmFunction;
  usesTime: boolean;
  usesParam: [boolean, boolean, boolean, boolean];
};

export function compile(
  source: string,
): { ok: true; value: Compiled } | { ok: false; error: SyntaxError } {
  const trimmed = source.trim();
  if (!trimmed) {
    return { ok: false, error: { message: "", at: 0 } };
  }
  try {
    const handle = new WasmFunction(trimmed);
    return {
      ok: true,
      value: {
        handle,
        usesTime: handle.usesTime(),
        usesParam: [0, 1, 2, 3].map((i) => handle.usesParam(i)) as [
          boolean,
          boolean,
          boolean,
          boolean,
        ],
      },
    };
  } catch (raw) {
    return { ok: false, error: parseError(raw) };
  }
}

function parseError(raw: unknown): SyntaxError {
  const text = typeof raw === "string" ? raw : String(raw);
  try {
    const parsed = JSON.parse(text) as SyntaxError;
    if (typeof parsed.message === "string") return parsed;
  } catch {
    /* not json, use the raw text */
  }
  return { message: text, at: 0 };
}

export { niceStep };

export const PALETTE = [
  "#5eead4",
  "#a78bfa",
  "#fbbf24",
  "#fb7185",
  "#60a5fa",
  "#4ade80",
  "#f472b6",
  "#facc15",
];

export const PARAM_NAMES = ["a", "b", "c", "d"] as const;
