import type { CSSProperties } from "react";
import { useWords } from "../i18n";
import { CARD, type Density, type Look, ROW, type TextSize } from "./look";

/// Mirrors `TextSize::zoom` and `measure::stands`, which the webview cannot ask for.
const ZOOM: Record<TextSize, number> = { small: 0.87, normal: 1, large: 1.13 };
const ICON: Record<Density, number> = { compact: 20, normal: 24, comfortable: 28 };

const SAMPLE = [
  { name: "Firefox", work: false },
  { name: "Chrome", work: true },
  { name: "Edge", work: false },
];

const CHOSEN = 1;

const HOST = "linkunbound.app";

type Shape = "list" | "cards" | "icons";

function shapeOf(look: Look): Shape {
  if (look.picker_style === "classic") return "list";
  if (look.picker_style === "compact") return "icons";
  return "cards";
}

function Icon({ at }: { at: number }) {
  return <i className={`look-icon tone-${at}`}>{SAMPLE[at]?.name[0]}</i>;
}

function Remember() {
  const t = useWords();
  return (
    <div className="look-remember">
      <small>{t("lookRemember")}</small>
      <div className="look-track">
        <span className="on">{t("lookOnce")}</span>
        <span>{t("lookSite")}</span>
        <span>{t("lookUrl")}</span>
      </div>
    </div>
  );
}

function List() {
  const t = useWords();
  return (
    <>
      <div className="look-head">
        <b>{HOST}</b>
        <span>{t("pickerPreview")}</span>
      </div>
      <div className="look-hair" />
      <div className="look-rows">
        {SAMPLE.map((b, at) => (
          <div key={b.name} className={at === CHOSEN ? "look-row lit" : "look-row"}>
            <kbd>{at + 1}</kbd>
            <Icon at={at} />
            <span className="look-name">{b.name}</span>
            {b.work && <em>{t("lookWork")}</em>}
          </div>
        ))}
      </div>
      <div className="look-hair" />
      <Remember />
    </>
  );
}

function Cards() {
  const t = useWords();
  return (
    <>
      <div className="look-ask">
        <b>{t("lookAsk")}</b>
        <span>
          {HOST} · {t("pickerPreview")}
        </span>
      </div>
      <div className="look-hair" />
      <div className="look-cards">
        {SAMPLE.map((b, at) => (
          <div key={b.name} className={at === CHOSEN ? "look-card lit" : "look-card"}>
            <Icon at={at} />
            <span className="look-name">
              <b>{b.name}</b>
              {b.work && <small>{t("lookWork")}</small>}
            </span>
            <kbd>{at + 1}</kbd>
          </div>
        ))}
      </div>
      <div className="look-hair" />
      <Remember />
    </>
  );
}

function Icons() {
  const t = useWords();
  const chosen = SAMPLE[CHOSEN];
  return (
    <>
      <div className="look-top">
        <b>{HOST}</b>
      </div>
      <div className="look-icons">
        {SAMPLE.map((b, at) => (
          <div key={b.name} className={at === CHOSEN ? "look-slot lit" : "look-slot"}>
            <Icon at={at} />
            <kbd>{at + 1}</kbd>
          </div>
        ))}
      </div>
      <div className="look-said">
        {chosen?.name}
        {chosen?.work && ` · ${t("lookWork")}`}
      </div>
    </>
  );
}

export default function LookPreview({ look, onPreview }: { look: Look; onPreview: () => void }) {
  const t = useWords();
  const shape = shapeOf(look);
  const measures = {
    "--zoom": ZOOM[look.text_size],
    "--row": `${ROW[look.density]}px`,
    "--card": `${CARD[look.density]}px`,
    "--icon": `${ICON[look.density]}px`,
  } as CSSProperties;
  return (
    <figure className="look-preview">
      <div className="look-desk">
        <div
          aria-hidden="true"
          data-shape={shape}
          className={`look-picker ${shape}${look.high_contrast ? " strong" : ""}`}
          style={measures}
        >
          {shape === "list" && <List />}
          {shape === "cards" && <Cards />}
          {shape === "icons" && <Icons />}
        </div>
      </div>
      <figcaption className="look-foot">
        <span>{t("lookPreviewSays")}</span>
        <button type="button" className="mild" onClick={onPreview}>
          {t("pickerPreview")}
        </button>
      </figcaption>
    </figure>
  );
}
