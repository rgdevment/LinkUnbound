import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { type Key, useSpoken, useWords } from "../i18n";
import { saidPlainly } from "../refusal";
import { refresh } from "../theme";
import LookPreview from "./LookPreview";
import {
  CARD,
  type Density,
  type Look,
  type PickerPosition,
  type PickerStyle,
  ROW,
  type TextSize,
  type Theme,
} from "./look";
import { Card, Choice, Line, Section, Switch } from "./parts";

const THEMES: { id: Theme; label: Key }[] = [
  { id: "system", label: "themeAuto" },
  { id: "light", label: "themeLight" },
  { id: "dark", label: "themeDark" },
];

const SIZES: { id: TextSize; label: Key }[] = [
  { id: "small", label: "sizeSmall" },
  { id: "normal", label: "sizeNormal" },
  { id: "large", label: "sizeLarge" },
];

const PICKERS: { id: PickerStyle; label: Key }[] = [
  { id: "bear", label: "pickerBear" },
  { id: "classic", label: "pickerClassic" },
  { id: "sheet", label: "pickerSheet" },
  { id: "compact", label: "pickerCompact" },
];

const DENSITIES: { id: Density; label: Key }[] = [
  { id: "compact", label: "densityCompact" },
  { id: "normal", label: "densityNormal" },
  { id: "comfortable", label: "densityComfortable" },
];

const POSITIONS: { id: PickerPosition; label: Key }[] = [
  { id: "pointer", label: "pickerByPointer" },
  { id: "center", label: "pickerCentered" },
];

function densityNote(look: Look, t: ReturnType<typeof useWords>): string {
  if (look.picker_style === "compact") return t("lookDensityNone");
  if (look.picker_style === "classic") return t("lookDensityRows", String(ROW[look.density]));
  return t("lookDensityCards", String(CARD[look.density]));
}

export default function Appearance() {
  const t = useWords();
  const language = useSpoken();
  const [look, setLook] = useState<Look | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [previewing, setPreviewing] = useState(false);

  useEffect(() => {
    void invoke<Look>("appearance_get").then(setLook).catch(noop);
  }, []);

  const preview = () => {
    setPreviewing(true);
    void invoke("picker_preview").catch(noop);
  };

  // An open preview is drawn again from scratch, so a larger size or density is placed anew.
  const settle = (asked: Promise<Look>, was: Look | null) =>
    void asked
      .then((next) => {
        setLook(next);
        setProblem(null);
        refresh();
        if (previewing) preview();
      })
      .catch((e: unknown) => {
        if (was) setLook(was);
        setProblem(saidPlainly(language, e));
      });

  const change = (patch: Partial<Look>) => {
    if (!look) return;
    const next = { ...look, ...patch };
    setLook(next);
    settle(invoke<Look>("appearance_set", { appearance: next }), look);
  };

  if (!look) return null;

  return (
    <>
      {problem && (
        <p className="rounded-md bg-danger/10 px-3 py-2 text-sm text-danger">{problem}</p>
      )}

      <LookPreview look={look} onPreview={preview} />

      <Section title={t("appTheme")}>
        <Card>
          <Line title={t("appTheme")}>
            <Choice
              options={THEMES}
              value={look.theme}
              label={t("appTheme")}
              onPick={(theme) => change({ theme })}
            />
          </Line>
          <Line title={t("lookContrast")} note={t("lookContrastNote")}>
            <Switch
              on={look.high_contrast}
              label={t("lookContrast")}
              onChange={(high_contrast) => change({ high_contrast })}
            />
          </Line>
        </Card>
      </Section>

      <Section title={t("lookText")}>
        <Card>
          <Line title={t("lookSize")} note={t("lookSizeNote")}>
            <Choice
              options={SIZES}
              value={look.text_size}
              label={t("lookSize")}
              onPick={(text_size) => change({ text_size })}
            />
          </Line>
        </Card>
      </Section>

      <Section title={t("appPicker")}>
        <Card>
          <Line title={t("pickerLook")}>
            <Choice
              options={PICKERS}
              value={look.picker_style}
              label={t("pickerLook")}
              onPick={(picker_style) => change({ picker_style })}
            />
          </Line>
          <Line title={t("lookDensity")} note={densityNote(look, t)}>
            <Choice
              options={DENSITIES}
              value={look.density}
              label={t("lookDensity")}
              disabled={look.picker_style === "compact"}
              onPick={(density) => change({ density })}
            />
          </Line>
          <Line title={t("appPickerPosition")} note={t("appPickerPositionNote")}>
            <Choice
              options={POSITIONS}
              value={look.picker_position}
              label={t("appPickerPosition")}
              onPick={(picker_position) => change({ picker_position })}
            />
          </Line>
        </Card>
      </Section>

      <div>
        <button
          type="button"
          onClick={() => settle(invoke<Look>("appearance_reset"), look)}
          className="rounded-md border border-edge px-3 py-1.5 text-sm"
        >
          {t("lookReset")}
        </button>
      </div>
    </>
  );
}

function noop() {}
