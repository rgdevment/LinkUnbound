import { useWords } from "../i18n";
import { Card, Line, Section } from "./parts";

const VERSION = "2.0.0";
const REPO = "https://github.com/rgdevment/LinkUnbound";
const COFFEE = "https://buymeacoffee.com/rgdevment";
const COPYPASTE = "https://github.com/rgdevment/CopyPaste";

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
  const t = useWords();

  return (
    <>
      <div className="flex items-center gap-3.5 rounded-lg border border-black/[0.08] bg-black/[0.015] px-4 py-3.5 dark:border-white/[0.08] dark:bg-white/[0.02]">
        <div className="min-w-0 flex-1">
          <p className="text-[13.5px] font-semibold">LinkUnbound {VERSION}</p>
          <p className="mt-0.5 text-[11.5px] text-neutral-500 dark:text-[#8B92A1]">
            {t("aboutTagline")}
          </p>
        </div>
      </div>

      <Section title={t("aboutProject")}>
        <Card>
          <Line title={t("aboutSource")} note={t("aboutSourceNote")}>
            <External href={REPO}>{t("open")}</External>
          </Line>
          <Line title={t("aboutLicense")} note={t("aboutLicenseNote")}>
            <External href={`${REPO}/blob/main/LICENSE`}>{t("open")}</External>
          </Line>
          <Line title={t("aboutIssue")} note={t("aboutIssueNote")}>
            <External href={`${REPO}/issues`}>{t("open")}</External>
          </Line>
        </Card>
      </Section>

      <Section title={t("aboutSponsor")}>
        <Card>
          <Line title={t("aboutSponsor")} note={t("aboutSponsorNote")}>
            <External href={COFFEE}>{t("open")}</External>
          </Line>
        </Card>
      </Section>

      <Section title={t("aboutOtherTools")}>
        <Card>
          <Line title={t("aboutCopyPaste")} note={t("aboutCopyPasteNote")}>
            <External href={COPYPASTE}>{t("open")}</External>
          </Line>
        </Card>
      </Section>
    </>
  );
}
