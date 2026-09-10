import { Card, Line, Section } from "./parts";

const VERSION = "2.0.0";
const REPO = "https://github.com/rgdevment/LinkUnbound";

function External({ href, children }: { href: string; children: string }) {
  return (
    <a
      href={href}
      target="_blank"
      rel="noreferrer"
      className="shrink-0 rounded-md px-3 py-1.5 text-[11.5px] text-[#2F62D8] dark:text-[#6E9BFF]"
    >
      {children}
    </a>
  );
}

export default function About() {
  return (
    <>
      <div className="flex items-center gap-3.5 rounded-lg border border-black/[0.08] bg-black/[0.015] px-4 py-3.5 dark:border-white/[0.08] dark:bg-white/[0.02]">
        <div className="min-w-0 flex-1">
          <p className="text-[13.5px] font-semibold">LinkUnbound {VERSION}</p>
          <p className="mt-0.5 text-[11.5px] text-neutral-500 dark:text-[#8B92A1]">
            Elige tú en qué navegador se abre cada enlace.
          </p>
        </div>
      </div>

      <Section title="Proyecto">
        <Card>
          <Line title="Código fuente" note="GPL-3.0 · las aportaciones son bienvenidas">
            <External href={REPO}>Abrir</External>
          </Line>
          <Line title="Informar de un problema" note="Errores, ideas y navegadores que no detecta">
            <External href={`${REPO}/issues`}>Abrir</External>
          </Line>
          <Line title="Apoyar el desarrollo" note="LinkUnbound es gratis y no lleva publicidad">
            <External href={`${REPO}#sponsors`}>Abrir</External>
          </Line>
        </Card>
      </Section>
    </>
  );
}
