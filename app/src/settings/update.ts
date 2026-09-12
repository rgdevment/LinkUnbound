import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";

export type Route = "store" | "brew" | "download";

export type Ready = {
  version: string;
  route: Route;
  package: string | null;
  installs: boolean;
};

export type Underway = { stage: "getting" | "installing"; far: number };

const LOOKS_AGAIN = 6 * 60 * 60 * 1000;

/// The look is throttled in Rust, so asking often costs a read of a local file rather than a
/// request: what this interval decides is only how soon a copy left open notices.
export function useUpdate() {
  const [ready, setReady] = useState<Ready | null>(null);
  const [step, setStep] = useState<Underway | null>(null);

  const look = useCallback(
    (nowPlease?: boolean) =>
      invoke<Ready | null>("update_ready", { nowPlease }).then((one) => {
        setReady(one);
        return one;
      }),
    [],
  );

  useEffect(() => {
    look().catch(() => {});
    const again = setInterval(() => {
      look().catch(() => {});
    }, LOOKS_AGAIN);
    return () => clearInterval(again);
  }, [look]);

  useEffect(() => {
    const stop = listen<Underway>("updating", (e) => setStep(e.payload));
    return () => {
      void stop.then((off) => off());
    };
  }, []);

  return { ready, step, look, settled: () => setStep(null) };
}
