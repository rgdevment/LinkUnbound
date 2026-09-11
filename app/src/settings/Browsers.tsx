import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { type Key, useWords } from "../i18n";
import { Card, Section, Switch } from "./parts";

type BrowserView = {
  id: string;
  name: string;
  exe: string;
  profiles: number;
  private: boolean;
  custom: boolean;
  hidden: boolean;
  icon: string | null;
  args: string[];
  private_flag: string | null;
  icon_path: string | null;
};

type Edit = {
  name: string;
  exe: string;
  args: string[];
  private_flag: string | null;
  icon_path: string | null;
};

const BLANK: Edit = { name: "", exe: "", args: [], private_flag: null, icon_path: null };

function describe(browser: BrowserView, t: (key: Key, ...values: string[]) => string): string {
  const parts = [
    browser.profiles === 0
      ? t("profilesNone")
      : browser.profiles === 1
        ? t("profilesOne")
        : t("profilesMany", String(browser.profiles)),
  ];
  parts.push(browser.private ? t("privateYes") : t("privateNo"));
  if (browser.hidden) parts.push(t("hiddenFromPicker"));
  return parts.join(" · ");
}

function Icon({ browser }: { browser: BrowserView }) {
  if (browser.icon) return <img src={browser.icon} alt="" className="h-[18px] w-[18px] shrink-0" />;
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      className="h-[18px] w-[18px] shrink-0 text-neutral-400 dark:text-[#646B7C]"
      aria-hidden="true"
    >
      <rect x="3" y="4" width="18" height="16" rx="3" />
      <path d="M3 9h18" />
    </svg>
  );
}

const FIELD =
  "rounded-md border border-black/[0.12] bg-transparent px-2.5 py-1.5 text-[12px] dark:border-white/[0.12]";

function Form({
  initial,
  onSave,
  onCancel,
}: {
  initial: Edit;
  onSave: (edit: Edit) => void;
  onCancel: () => void;
}) {
  const t = useWords();
  const [name, setName] = useState(initial.name);
  const [exe, setExe] = useState(initial.exe);
  const [args, setArgs] = useState(initial.args.join(" "));
  const [priv, setPriv] = useState(initial.private_flag ?? "");
  const [icon, setIcon] = useState(initial.icon_path ?? "");

  return (
    <form
      className="flex flex-col gap-2 border-black/[0.08] px-3.5 py-3 not-first:border-t dark:border-white/[0.08]"
      onSubmit={(e) => {
        e.preventDefault();
        onSave({
          name,
          exe,
          args: args.split(" ").filter(Boolean),
          private_flag: priv || null,
          icon_path: icon || null,
        });
      }}
    >
      <input
        value={name}
        onChange={(e) => setName(e.target.value)}
        placeholder={t("fieldName")}
        aria-label={t("fieldName")}
        required
        className={FIELD}
      />
      <input
        value={exe}
        onChange={(e) => setExe(e.target.value)}
        placeholder={t("fieldExe")}
        aria-label={t("fieldExe")}
        required
        className={FIELD}
      />
      <input
        value={args}
        onChange={(e) => setArgs(e.target.value)}
        placeholder={t("fieldArgs")}
        aria-label={t("fieldArgsLabel")}
        className={FIELD}
      />
      <input
        value={priv}
        onChange={(e) => setPriv(e.target.value)}
        placeholder={t("fieldPrivate")}
        aria-label={t("fieldPrivateLabel")}
        className={FIELD}
      />
      <p className="-mt-1 text-[10.5px] text-neutral-500 dark:text-[#8B92A1]">
        {t("fieldPrivateHint")}
      </p>
      <input
        value={icon}
        onChange={(e) => setIcon(e.target.value)}
        placeholder={t("fieldIcon")}
        aria-label={t("fieldIconLabel")}
        className={FIELD}
      />
      <div className="flex gap-2">
        <button
          type="submit"
          className="rounded-md bg-[#2F62D8] px-3 py-1.5 text-[11.5px] font-medium text-white dark:bg-[#6E9BFF] dark:text-[#12141B]"
        >
          {t("save")}
        </button>
        <button
          type="button"
          onClick={onCancel}
          className="rounded-md px-3 py-1.5 text-[11.5px] text-neutral-500 dark:text-[#8B92A1]"
        >
          {t("cancel")}
        </button>
      </div>
    </form>
  );
}

const GHOST =
  "grid h-6 w-6 shrink-0 place-items-center rounded text-neutral-500 hover:bg-black/[0.06] disabled:opacity-25 dark:text-[#8B92A1] dark:hover:bg-white/[0.08]";

export default function Browsers() {
  const t = useWords();
  const [list, setList] = useState<BrowserView[] | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [editing, setEditing] = useState<string | null>(null);

  const run = useCallback((command: string, params: Record<string, unknown> = {}) => {
    void invoke<BrowserView[]>(command, params)
      .then((next) => {
        setList(next);
        setProblem(null);
        setEditing(null);
      })
      .catch((e: unknown) => setProblem(String(e)));
  }, []);

  useEffect(() => run("browsers_list"), [run]);

  if (list === null) return null;

  const move = (id: string, step: number) => {
    const order = list.map((b) => b.id);
    const at = order.indexOf(id);
    const to = at + step;
    if (to < 0 || to >= order.length) return;
    [order[at], order[to]] = [order[to], order[at]];
    run("browsers_reorder", { ids: order });
  };

  const detected = list.filter((b) => !b.custom);
  const mine = list.filter((b) => b.custom);

  return (
    <>
      {problem && (
        <p className="rounded-md bg-[#C0362F]/10 px-3 py-2 text-[11.5px] text-[#C0362F] dark:bg-[#FF8A85]/10 dark:text-[#FF8A85]">
          {problem}
        </p>
      )}

      <Section title={t("browsersDetected")}>
        <Card>
          {detected.map((b) => (
            <div
              key={b.id}
              className="group flex items-center gap-3 border-black/[0.08] px-3.5 py-3 not-first:border-t dark:border-white/[0.08]"
            >
              <Icon browser={b} />
              <div className="min-w-0 flex-1">
                <p className={`text-[12.5px] ${b.hidden ? "opacity-55" : ""}`}>{b.name}</p>
                <p className="mt-px text-[11px] text-neutral-500 dark:text-[#8B92A1]">
                  {describe(b, t)}
                </p>
              </div>
              <span className="flex shrink-0 gap-px opacity-0 transition group-hover:opacity-100 focus-within:opacity-100">
                <button
                  type="button"
                  aria-label={t("browserUp", b.name)}
                  disabled={list[0]?.id === b.id}
                  onClick={() => move(b.id, -1)}
                  className={GHOST}
                >
                  ↑
                </button>
                <button
                  type="button"
                  aria-label={t("browserDown", b.name)}
                  disabled={list.at(-1)?.id === b.id}
                  onClick={() => move(b.id, 1)}
                  className={GHOST}
                >
                  ↓
                </button>
                <button
                  type="button"
                  aria-label={t("browserDuplicate", b.name)}
                  onClick={() => run("browsers_duplicate", { id: b.id })}
                  className={GHOST}
                >
                  ⧉
                </button>
              </span>
              <Switch
                on={!b.hidden}
                label={t("browserShow", b.name)}
                onChange={(next) => run("browsers_set_hidden", { id: b.id, hidden: !next })}
              />
            </div>
          ))}
          {detected.length === 0 && (
            <p className="px-3.5 py-4 text-center text-[12px] text-neutral-500 dark:text-[#8B92A1]">
              {t("browsersNone")}
            </p>
          )}
        </Card>
      </Section>

      <Section title={t("browsersMine")}>
        <Card>
          {mine.map((b) =>
            editing === b.id ? (
              <Form
                key={b.id}
                initial={b}
                onCancel={() => setEditing(null)}
                onSave={(edit) => run("browsers_update", { id: b.id, edit })}
              />
            ) : (
              <div
                key={b.id}
                className="group flex items-center gap-3 border-black/[0.08] px-3.5 py-3 not-first:border-t dark:border-white/[0.08]"
              >
                <Icon browser={b} />
                <div className="min-w-0 flex-1">
                  <p className={`truncate text-[12.5px] ${b.hidden ? "opacity-55" : ""}`}>
                    {b.name}
                  </p>
                  <p className="mt-px truncate text-[11px] text-neutral-500 dark:text-[#8B92A1]">
                    {b.exe}
                  </p>
                </div>
                <span className="flex shrink-0 gap-px opacity-0 transition group-hover:opacity-100 focus-within:opacity-100">
                  <button
                    type="button"
                    aria-label={t("browserEdit", b.name)}
                    onClick={() => setEditing(b.id)}
                    className={GHOST}
                  >
                    ✎
                  </button>
                  <button
                    type="button"
                    aria-label={t("browserDuplicate", b.name)}
                    onClick={() => run("browsers_duplicate", { id: b.id })}
                    className={GHOST}
                  >
                    ⧉
                  </button>
                  <button
                    type="button"
                    aria-label={t("browserRemove", b.name)}
                    onClick={() => run("browsers_remove", { id: b.id })}
                    className="grid h-6 w-6 shrink-0 place-items-center rounded text-neutral-500 hover:bg-[#C0362F]/10 hover:text-[#C0362F] dark:text-[#8B92A1] dark:hover:bg-[#FF8A85]/10 dark:hover:text-[#FF8A85]"
                  >
                    ✕
                  </button>
                </span>
                <Switch
                  on={!b.hidden}
                  label={t("browserShow", b.name)}
                  onChange={(next) => run("browsers_set_hidden", { id: b.id, hidden: !next })}
                />
              </div>
            ),
          )}

          {editing === "new" ? (
            <Form
              initial={BLANK}
              onCancel={() => setEditing(null)}
              onSave={(edit) => run("browsers_add", { edit })}
            />
          ) : (
            <button
              type="button"
              onClick={() => setEditing("new")}
              className="w-full border-black/[0.08] px-3.5 py-3 text-left text-[12px] text-[#2F62D8] not-first:border-t dark:border-white/[0.08] dark:text-[#6E9BFF]"
            >
              {t("browsersAdd")}
            </button>
          )}
        </Card>
      </Section>
    </>
  );
}
