const PATHS = {
  search: "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zM20 20l-3.5-3.5",
  pencil: "M16.5 3.5a2.1 2.1 0 0 1 3 3L8 18l-4 1 1-4z",
  trash: "M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3",
  warn: "M12 4 2.8 19.5h18.4zM12 10v4.5M12 17.2v.1",
  mask: "M2.5 9.5c3-2.2 6.5-2.2 9.5 0 3-2.2 6.5-2.2 9.5 0 0 5-3 7-5 7-2.2 0-3-2.3-4.5-2.3S9.7 16.5 7.5 16.5c-2 0-5-2-5-7z",
  x: "M6 6l12 12M18 6 6 18",
  check: "m5 12.5 4.5 4.5L19 7.5",
  chev: "m7 10 5 5 5-5",
  info: "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM12 16v-4.5M12 8v.1",
  flask: "M9 3h6M10 3v6L4.5 19a1.3 1.3 0 0 0 1.1 2h12.8a1.3 1.3 0 0 0 1.1-2L14 9V3M7.5 14h9",
  plus: "M12 5v14M5 12h14",
  rules: "M4 6h16M4 12h11M4 18h7",
  window:
    "M5.5 4.5h13A2.5 2.5 0 0 1 21 7v10a2.5 2.5 0 0 1-2.5 2.5h-13A2.5 2.5 0 0 1 3 17V7a2.5 2.5 0 0 1 2.5-2.5zM3 9h18",
} as const;

export type GlyphName = keyof typeof PATHS;

export default function Glyph({ name, size = 14 }: Readonly<{ name: GlyphName; size?: number }>) {
  return (
    <svg
      viewBox="0 0 24 24"
      width={size}
      height={size}
      aria-hidden="true"
      className="shrink-0 fill-none stroke-current"
      strokeWidth={1.8}
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d={PATHS[name]} />
    </svg>
  );
}

/// A browser's own icon, or a quiet square where the system gave none.
export function BrowserIcon({ src, size = 16 }: Readonly<{ src: string | null; size?: number }>) {
  return src ? (
    <img src={src} alt="" width={size} height={size} className="shrink-0" />
  ) : (
    <span
      aria-hidden="true"
      style={{ width: size, height: size }}
      className="shrink-0 rounded bg-(--tile)"
    />
  );
}

/// An app's own icon, or a plain window where the system gave none.
export function AppIcon({ src }: Readonly<{ src: string | null }>) {
  return src ? (
    <img src={src} alt="" width={16} height={16} className="shrink-0" />
  ) : (
    <span aria-hidden="true" className="text-muted">
      <Glyph name="window" size={16} />
    </span>
  );
}
