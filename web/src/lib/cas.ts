export type CasResult = { ok: boolean; text: string };

type Bridge = {
  version: string;
  simplify(source: string, variable: string): CasResult;
  derivative(source: string, variable: string): CasResult;
  integral(source: string, variable: string): CasResult;
  solve(source: string, variable: string): CasResult;
  taylor(source: string, variable: string, about: number, order: number): CasResult;
};

declare global {
  // eslint-disable-next-line no-var
  var kosmosCas: Bridge | undefined;
}

let loading: Promise<Bridge | null> | null = null;

export function algebra(): Promise<Bridge | null> {
  if (loading) return loading;

  loading = new Promise((resolve) => {
    if (globalThis.kosmosCas) {
      resolve(globalThis.kosmosCas);
      return;
    }
    const tag = document.createElement("script");
    tag.src = `${import.meta.env.BASE_URL}cas.js`;
    tag.async = true;
    tag.onload = () => resolve(globalThis.kosmosCas ?? null);
    tag.onerror = () => resolve(null);
    document.head.append(tag);
  });

  return loading;
}

export const UNAVAILABLE =
  "the algebra module could not be loaded — build it with cas/build.sh";
