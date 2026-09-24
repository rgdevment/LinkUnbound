function Stroke({ children }: { children: React.ReactNode }) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      {children}
    </svg>
  );
}

export function Info() {
  return (
    <Stroke>
      <circle cx="12" cy="12" r="10" />
      <path d="M12 16v-4M12 8h.01" />
    </Stroke>
  );
}

export function Key() {
  return (
    <Stroke>
      <path d="M21 2l-2 2m-7.6 7.6a5 5 0 1 1-7.1 7.1 5 5 0 0 1 7.1-7.1zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3" />
    </Stroke>
  );
}

export function Code() {
  return (
    <Stroke>
      <path d="M16 18l6-6-6-6M8 6l-6 6 6 6" />
    </Stroke>
  );
}

export function Gift() {
  return (
    <Stroke>
      <path d="M20 12v10H4V12M2 7h20v5H2zM12 22V7" />
      <path d="M12 7H7.5a2.5 2.5 0 0 1 0-5C11 2 12 7 12 7zM12 7h4.5a2.5 2.5 0 0 0 0-5C13 2 12 7 12 7z" />
    </Stroke>
  );
}

export function CloudOff() {
  return (
    <Stroke>
      <path d="M22.6 16.9A5 5 0 0 0 18 10h-1.3A8 8 0 0 0 4 7.4M1 1l22 22M16 16H8a4 4 0 0 1-.6-8" />
    </Stroke>
  );
}
