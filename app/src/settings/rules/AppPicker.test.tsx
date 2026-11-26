import { render, renderHook, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useRef, useState } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useWords } from "../../i18n";
import AppPicker, { type SeenApp, seenSince } from "./AppPicker";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const LABEL = "Busca una aplicación o escribe su nombre";
const NOON = new Date("2026-11-26T12:00:00Z");
const ICON = "data:image/png;base64,iVBORw0KGgo=";

const seconds = (when: string) => new Date(when).getTime() / 1000;

function seen(name: string, over: Partial<SeenApp> = {}): SeenApp {
  return {
    name,
    links: 1,
    last_seen: seconds("2026-11-26T09:00:00Z"),
    icon: null,
    rule_id: null,
    ...over,
  };
}

function answering(apps: SeenApp[]) {
  invoke.mockImplementation(() => Promise.resolve(apps));
}

function Harness({
  first = "",
  editingId,
  startOpen = false,
  onSubmit = () => {},
}: Readonly<{
  first?: string;
  editingId?: string;
  startOpen?: boolean;
  onSubmit?: () => void;
}>) {
  const [value, setValue] = useState(first);
  const box = useRef<HTMLInputElement>(null);
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit();
      }}
    >
      <AppPicker
        label={LABEL}
        value={value}
        onChange={setValue}
        editingId={editingId}
        startOpen={startOpen}
        inputRef={box}
      />
      <button type="button">elsewhere</button>
    </form>
  );
}

const combobox = () => screen.getByRole("combobox", { name: LABEL });
const options = () => within(screen.getByRole("listbox")).getAllByRole("option");
const activeName = () => {
  const id = combobox().getAttribute("aria-activedescendant");
  return id ? (document.getElementById(id)?.textContent ?? "") : null;
};

describe("app picker", () => {
  beforeEach(() => {
    invoke.mockReset();
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(NOON);
  });
  afterEach(() => vi.useRealTimers());

  describe("what it says about when an app was seen", () => {
    const said = (when: string, now: string) => {
      const { result } = renderHook(() => useWords());
      return seenSince(result.current, seconds(when), new Date(now));
    };

    it("counts calendar days, so ten minutes past midnight is already yesterday", () => {
      expect(said("2026-11-25T23:50:00Z", "2026-11-26T00:05:00Z")).toBe("ayer");
      expect(said("2026-11-26T00:01:00Z", "2026-11-26T00:05:00Z")).toBe("hoy");
      expect(said("2026-11-26T00:00:00Z", "2026-11-26T23:59:59Z")).toBe("hoy");
      expect(said("2026-11-25T00:00:00Z", "2026-11-26T23:59:59Z")).toBe("ayer");
    });

    it("moves from days to weeks between the thirteenth day and the fourteenth", () => {
      expect(said("2026-11-24T12:00:00Z", "2026-11-26T12:00:00Z")).toBe("hace 2 días");
      expect(said("2026-11-13T12:00:00Z", "2026-11-26T12:00:00Z")).toBe("hace 13 días");
      expect(said("2026-11-12T12:00:00Z", "2026-11-26T12:00:00Z")).toBe("hace 2 semanas");
      expect(said("2026-11-06T12:00:00Z", "2026-11-26T12:00:00Z")).toBe("hace 2 semanas");
      expect(said("2026-11-05T12:00:00Z", "2026-11-26T12:00:00Z")).toBe("hace 3 semanas");
    });

    it("never says a date in the future is anything but today", () => {
      expect(said("2026-11-27T09:00:00Z", "2026-11-26T12:00:00Z")).toBe("hoy");
    });
  });

  describe("the list", () => {
    it("shows the empty state, and no group title, when nothing has been seen", async () => {
      answering([]);
      render(<Harness startOpen />);
      expect(
        await screen.findByText("El selector todavía no ha visto enlaces de ninguna aplicación"),
      ).toBeInTheDocument();
      expect(
        screen.getByText(/Abre un enlace desde la aplicación que quieras/),
      ).toBeInTheDocument();
      expect(screen.queryByText("Vistas por el selector")).toBeNull();
      expect(screen.queryAllByRole("option")).toHaveLength(0);
    });

    it("is the empty state too when the command fails", async () => {
      invoke.mockRejectedValue("nope");
      render(<Harness startOpen />);
      expect(
        await screen.findByText("El selector todavía no ha visto enlaces de ninguna aplicación"),
      ).toBeInTheDocument();
    });

    it("types a name that is not there as the only way forward in an empty list", async () => {
      answering([]);
      render(<Harness startOpen />);
      await screen.findByRole("listbox");
      await userEvent.type(combobox(), "zoom");
      const use = screen.getByRole("option", { name: /Usar «zoom»/ });
      expect(use).toHaveTextContent("Solo se aplica si el selector muestra «Desde zoom»");
      await userEvent.click(use);
      expect(combobox()).toHaveValue("zoom");
      expect(screen.queryByRole("listbox")).toBeNull();
    });

    it("lists the apps in the order it is given, under the group title, with what is known of each", async () => {
      answering([
        seen("slack", { links: 3 }),
        seen("olk", { links: 1, last_seen: seconds("2026-11-25T20:00:00Z") }),
        seen("teams", { links: 12, last_seen: seconds("2026-11-23T20:00:00Z") }),
        seen("zoom", { links: 2, last_seen: seconds("2026-10-20T20:00:00Z") }),
        seen("notes", { links: 0, last_seen: null }),
      ]);
      render(<Harness startOpen />);
      expect(await screen.findByText("Vistas por el selector")).toBeInTheDocument();
      const rows = options();
      expect(rows.map((r) => r.textContent)).toEqual([
        "slack3 enlaces · hoy",
        "olk1 enlace · ayer",
        "teams12 enlaces · hace 3 días",
        "zoom2 enlaces · hace 5 semanas",
        "notesnombrada en una regla",
      ]);
    });

    it("draws the icon of an app that has one and a generic window for one that has not", async () => {
      answering([seen("slack", { icon: ICON }), seen("olk")]);
      render(<Harness startOpen />);
      await screen.findByRole("listbox");
      const [slack, olk] = options();
      expect(slack.querySelector("img")).toHaveAttribute("src", ICON);
      expect(slack.querySelector("svg")).toBeNull();
      expect(olk.querySelector("img")).toBeNull();
      expect(olk.querySelector("svg")).not.toBeNull();
    });

    it("filters by what is typed, in any case, and marks the part that matched", async () => {
      answering([seen("slack"), seen("ms-teams"), seen("olk")]);
      render(<Harness startOpen />);
      await screen.findByRole("listbox");
      await userEvent.type(combobox(), "TEA");
      const rows = options().filter((o) => o.textContent?.includes("enlace"));
      expect(rows).toHaveLength(1);
      expect(rows[0]).toHaveTextContent("ms-teams");
      expect(rows[0].querySelector("mark")).toHaveTextContent("tea");
    });

    it("says no app matches, and offers the typed name, when the filter leaves nothing", async () => {
      answering([seen("slack")]);
      render(<Harness startOpen />);
      await screen.findByRole("listbox");
      await userEvent.type(combobox(), "xyz");
      expect(screen.getByText("Ninguna aplicación vista coincide")).toBeInTheDocument();
      expect(screen.getByRole("option", { name: /Usar «xyz»/ })).toBeInTheDocument();
      expect(screen.queryByRole("option", { name: /slack/ })).toBeNull();
    });

    it("offers «Usar» for a part of a name but not for a name that is already there", async () => {
      answering([seen("slack")]);
      render(<Harness startOpen />);
      await screen.findByRole("listbox");
      await userEvent.type(combobox(), "sla");
      expect(screen.getByRole("option", { name: /Usar «sla»/ })).toBeInTheDocument();
      await userEvent.type(combobox(), "ck");
      expect(screen.queryByRole("option", { name: /Usar/ })).toBeNull();
      await userEvent.clear(combobox());
      await userEvent.type(combobox(), " SLACK ");
      expect(screen.queryByRole("option", { name: /Usar/ })).toBeNull();
    });

    it("puts «Usar» after the apps, apart from them", async () => {
      answering([seen("slack"), seen("slick")]);
      render(<Harness startOpen />);
      await screen.findByRole("listbox");
      await userEvent.type(combobox(), "sl");
      const names = options().map((o) => o.textContent);
      expect(names).toHaveLength(3);
      expect(names[2]).toContain("Usar «sl»");
    });

    it("marks the app that is the value with a check, and no other", async () => {
      answering([seen("slack", { icon: ICON }), seen("olk", { icon: ICON })]);
      render(<Harness first="olk" startOpen />);
      await screen.findByRole("listbox");
      const [slack, olk] = options();
      expect(olk).toHaveAttribute("aria-selected", "true");
      expect(olk.querySelector("svg")).not.toBeNull();
      expect(slack).toHaveAttribute("aria-selected", "false");
      expect(slack.querySelector("svg")).toBeNull();
    });

    it("shows the icon of the chosen app in the box", async () => {
      answering([seen("slack", { icon: ICON })]);
      const { container } = render(<Harness first="Slack" />);
      await vi.waitFor(() => expect(container.querySelector("img")).toHaveAttribute("src", ICON));
    });
  });

  describe("an app that already has its rule", () => {
    const apps = () => [
      seen("slack", { rule_id: "any@slack" }),
      seen("olk"),
      seen("teams", { rule_id: "any@teams" }),
    ];

    it("is dimmed, says so instead of its counts, and cannot be picked", async () => {
      answering(apps());
      render(<Harness startOpen />);
      await screen.findByRole("listbox");
      const slack = options()[0];
      expect(slack).toHaveAttribute("aria-disabled", "true");
      expect(slack).toHaveClass("opacity-50");
      expect(slack).toHaveTextContent("ya tiene regla");
      expect(slack).not.toHaveTextContent("enlace");
      await userEvent.click(slack);
      expect(combobox()).toHaveValue("");
      expect(screen.queryByRole("listbox")).toBeNull();
    });

    it("is the one being edited when the rule is its own, selected and not dimmed", async () => {
      answering(apps());
      render(<Harness first="teams" editingId="any@teams" startOpen />);
      await screen.findByRole("listbox");
      const [slack, , teams] = options();
      expect(teams).not.toHaveAttribute("aria-disabled");
      expect(teams).not.toHaveClass("opacity-50");
      expect(teams).toHaveAttribute("aria-selected", "true");
      expect(teams).toHaveTextContent("1 enlace · hoy");
      expect(slack).toHaveAttribute("aria-disabled", "true");
    });

    it("is dimmed for an edited rule that is not the one that holds it", async () => {
      answering(apps());
      render(<Harness editingId="any@other" startOpen />);
      await screen.findByRole("listbox");
      expect(options()[2]).toHaveAttribute("aria-disabled", "true");
    });
  });

  describe("the keyboard", () => {
    it("lights the first app a person can pick, not one that is taken", async () => {
      answering([seen("slack", { rule_id: "any@slack" }), seen("olk")]);
      render(<Harness startOpen />);
      await screen.findByRole("listbox");
      expect(activeName()).toContain("olk");
    });

    it("walks the apps with the arrows, jumps the taken ones and stops at the ends", async () => {
      answering([seen("a-first"), seen("b-taken", { rule_id: "any@b" }), seen("c-third")]);
      render(<Harness startOpen />);
      await screen.findByRole("listbox");
      combobox().focus();
      expect(activeName()).toContain("a-first");
      await userEvent.keyboard("{ArrowDown}");
      expect(activeName()).toContain("c-third");
      await userEvent.keyboard("{ArrowDown}");
      expect(activeName()).toContain("c-third");
      await userEvent.keyboard("{ArrowUp}");
      expect(activeName()).toContain("a-first");
      await userEvent.keyboard("{ArrowUp}");
      expect(activeName()).toContain("a-first");
    });

    it("walks on to «Usar» after the last app", async () => {
      answering([seen("slack")]);
      render(<Harness startOpen />);
      await screen.findByRole("listbox");
      await userEvent.type(combobox(), "sl");
      await userEvent.keyboard("{ArrowDown}");
      expect(activeName()).toContain("Usar «sl»");
    });

    it("starts from the app that is the value", async () => {
      answering([seen("slack"), seen("olk"), seen("teams")]);
      render(<Harness first="olk" startOpen />);
      await screen.findByRole("listbox");
      expect(activeName()).toContain("olk");
    });

    it("picks the lit app with Enter and does not send the form", async () => {
      answering([seen("slack"), seen("olk")]);
      const onSubmit = vi.fn();
      render(<Harness startOpen onSubmit={onSubmit} />);
      await screen.findByRole("listbox");
      combobox().focus();
      await userEvent.keyboard("{ArrowDown}{Enter}");
      expect(combobox()).toHaveValue("olk");
      expect(screen.queryByRole("listbox")).toBeNull();
      expect(onSubmit).not.toHaveBeenCalled();
    });

    it("picks what was typed with Enter when no app matches", async () => {
      answering([seen("slack")]);
      const onSubmit = vi.fn();
      render(<Harness startOpen onSubmit={onSubmit} />);
      await screen.findByRole("listbox");
      await userEvent.type(combobox(), "zoom{Enter}");
      expect(combobox()).toHaveValue("zoom");
      expect(onSubmit).not.toHaveBeenCalled();
    });

    it("closes only the list with Escape, and the key goes no further", async () => {
      answering([seen("slack")]);
      const outer = vi.fn();
      render(
        // biome-ignore lint/a11y/noStaticElementInteractions: stands for the form's own Escape
        <div onKeyDown={(e) => e.key === "Escape" && outer()}>
          <Harness startOpen />
        </div>,
      );
      await screen.findByRole("listbox");
      combobox().focus();
      await userEvent.keyboard("{Escape}");
      expect(screen.queryByRole("listbox")).toBeNull();
      expect(outer).not.toHaveBeenCalled();
      await userEvent.keyboard("{Escape}");
      expect(outer).toHaveBeenCalledTimes(1);
    });

    it("opens a closed list with the arrow down, and closes it with Tab", async () => {
      answering([seen("slack")]);
      render(<Harness />);
      combobox().focus();
      expect(screen.queryByRole("listbox")).toBeNull();
      await userEvent.keyboard("{ArrowDown}");
      expect(await screen.findByRole("listbox")).toBeInTheDocument();
      expect(combobox()).toHaveAttribute("aria-expanded", "true");
      await userEvent.keyboard("{Tab}");
      expect(screen.queryByRole("listbox")).toBeNull();
      expect(combobox()).toHaveAttribute("aria-expanded", "false");
    });

    it("does nothing with Enter when the list is closed beyond leaving the form to it", async () => {
      answering([seen("slack")]);
      const onSubmit = vi.fn();
      render(<Harness first="slack" onSubmit={onSubmit} />);
      combobox().focus();
      await userEvent.keyboard("{Enter}");
      expect(onSubmit).toHaveBeenCalledTimes(1);
    });
  });

  describe("opening", () => {
    const asked = () => invoke.mock.calls.filter(([c]) => c === "apps_seen").length;

    it("starts closed and opens on a click in the box", async () => {
      answering([seen("slack")]);
      render(<Harness />);
      await vi.waitFor(() => expect(asked()).toBe(1));
      expect(screen.queryByRole("listbox")).toBeNull();
      await userEvent.click(combobox());
      expect(await screen.findByRole("listbox")).toBeInTheDocument();
    });

    it("opens when the person starts typing", async () => {
      answering([seen("slack")]);
      render(<Harness />);
      await userEvent.type(combobox(), "s");
      expect(await screen.findByRole("listbox")).toBeInTheDocument();
    });

    it("asks again every time it opens, so a link opened a moment ago is there", async () => {
      answering([seen("slack")]);
      render(<Harness />);
      await vi.waitFor(() => expect(asked()).toBe(1));
      await userEvent.click(combobox());
      await vi.waitFor(() => expect(asked()).toBe(2));
      answering([seen("slack"), seen("newcomer")]);
      await userEvent.keyboard("{Escape}");
      await userEvent.click(combobox());
      await vi.waitFor(() => expect(asked()).toBe(3));
      expect(await screen.findByRole("option", { name: /newcomer/ })).toBeInTheDocument();
    });

    it("drops an older answer that lands after the one asked later", async () => {
      let late: (apps: SeenApp[]) => void = () => {};
      invoke
        .mockImplementationOnce(() => new Promise<SeenApp[]>((done) => (late = done)))
        .mockImplementationOnce(() => Promise.resolve([seen("fresh")]));
      render(<Harness />);
      await userEvent.click(combobox());
      expect(await screen.findByRole("option", { name: /fresh/ })).toBeInTheDocument();
      late([seen("stale")]);
      await vi.waitFor(() => expect(asked()).toBe(2));
      await new Promise((done) => setTimeout(done, 30));
      expect(screen.getByRole("option", { name: /fresh/ })).toBeInTheDocument();
      expect(screen.queryByRole("option", { name: /stale/ })).toBeNull();
    });

    it("opens and closes with the chevron and gives the focus back to the box", async () => {
      answering([seen("slack")]);
      const { container } = render(<Harness />);
      const chevron = container.querySelector("button[aria-hidden='true']") as HTMLElement;
      await userEvent.click(chevron);
      expect(await screen.findByRole("listbox")).toBeInTheDocument();
      expect(combobox()).toHaveFocus();
      await userEvent.click(chevron);
      expect(screen.queryByRole("listbox")).toBeNull();
    });

    it("closes when the person clicks somewhere else, and not when they click inside", async () => {
      answering([seen("slack")]);
      render(<Harness startOpen />);
      await screen.findByRole("listbox");
      await userEvent.click(screen.getByText("Vistas por el selector"));
      expect(screen.getByRole("listbox")).toBeInTheDocument();
      await userEvent.click(screen.getByRole("button", { name: "elsewhere" }));
      expect(screen.queryByRole("listbox")).toBeNull();
    });

    it("picks an app with a click and leaves the list closed with the name in the box", async () => {
      answering([seen("slack"), seen("olk")]);
      render(<Harness startOpen />);
      await screen.findByRole("listbox");
      await userEvent.click(options()[1]);
      expect(combobox()).toHaveValue("olk");
      expect(screen.queryByRole("listbox")).toBeNull();
    });

    it("lights the app under the pointer, but not one that is taken", async () => {
      answering([seen("slack", { rule_id: "any@slack" }), seen("olk"), seen("teams")]);
      render(<Harness startOpen />);
      await screen.findByRole("listbox");
      await userEvent.hover(options()[2]);
      expect(activeName()).toContain("teams");
      await userEvent.hover(options()[0]);
      expect(activeName()).toContain("teams");
    });
  });
});
