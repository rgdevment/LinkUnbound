import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import FontPicker, { matching } from "./FontPicker";
import type { Fonts } from "./look";

const FONTS: Fonts = {
  system: "Segoe UI",
  families: [
    { name: "Arial", label: "Arial" },
    { name: "Courier New", label: "Courier New" },
    { name: "Yu Gothic", label: "Yu Gothic Médium" },
    { name: 'Odd"Name', label: 'Odd"Name' },
  ],
};

const SYSTEM = "Sistema (Segoe UI) · predeterminada";

function mount(chosen: string | null = null) {
  const onPick = vi.fn();
  render(<FontPicker label="Fuente" fonts={FONTS} chosen={chosen} onPick={onPick} />);
  return onPick;
}

const opener = (current: string) => screen.getByRole("button", { name: `Fuente: ${current}` });
const search = () => screen.getByRole("combobox", { name: "Buscar una fuente instalada" });
const optionNames = () => screen.queryAllByRole("option").map((one) => one.textContent);
const drop = () => screen.getByRole("listbox").parentElement as HTMLElement;

async function open(chosen: string | null = null) {
  const onPick = mount(chosen);
  await userEvent.click(opener(chosen ?? SYSTEM));
  return onPick;
}

function buttonAt(top: number, bottom: number) {
  return vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
    top,
    bottom,
    right: 700,
    left: 500,
    width: 200,
    height: bottom - top,
    x: 500,
    y: top,
    toJSON() {},
  });
}

describe("matching", () => {
  const all = FONTS.families;

  it("returns everything for a blank query", () => {
    expect(matching(all, "   ")).toEqual(all);
  });

  it("finds by the label without caring for accents or case", () => {
    expect(matching(all, "MEDIUM").map((one) => one.name)).toEqual(["Yu Gothic"]);
    expect(matching(all, "médium").map((one) => one.name)).toEqual(["Yu Gothic"]);
  });

  it("finds by the English name when the label is in another script", () => {
    expect(matching([{ name: "Meiryo", label: "メイリオ" }], "meir")).toHaveLength(1);
  });

  it("finds nothing for a name nobody has", () => {
    expect(matching(all, "zzz")).toEqual([]);
  });
});

describe("the font picker", () => {
  beforeAll(() => {
    // jsdom has no layout, so no scrolling.
    Element.prototype.scrollIntoView = vi.fn();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("shows System as the current one, closed", () => {
    mount();
    expect(opener(SYSTEM)).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  it("lists System first and checked, then the families", async () => {
    await open();
    expect(optionNames()).toEqual([SYSTEM, "Arial", "Courier New", "Yu Gothic Médium", 'Odd"Name']);
    expect(screen.getByRole("option", { name: SYSTEM })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("option", { name: "Arial" })).toHaveAttribute("aria-selected", "false");
  });

  it("checks the family that is in force", async () => {
    await open("Courier New");
    expect(screen.getByRole("option", { name: "Courier New" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByRole("option", { name: SYSTEM })).toHaveAttribute("aria-selected", "false");
  });

  it("falls back to System for a family that is no longer installed", () => {
    mount("Gone Sans");
    expect(opener(SYSTEM)).toBeInTheDocument();
  });

  it("writes each option and the button in its own font", async () => {
    await open("Courier New");
    expect(screen.getByRole("option", { name: "Arial" })).toHaveStyle({ fontFamily: '"Arial"' });
    expect(screen.getByRole("option", { name: "Yu Gothic Médium" })).toHaveStyle({
      fontFamily: '"Yu Gothic"',
    });
    expect(opener("Courier New")).toHaveStyle({ fontFamily: '"Courier New"' });
    expect(screen.getByRole("option", { name: SYSTEM }).style.fontFamily).toBe("");
  });

  it("filters as you type, by label or name, ignoring accents", async () => {
    await open();
    await userEvent.type(search(), "medium");
    expect(optionNames()).toEqual(["Yu Gothic Médium"]);
    await userEvent.clear(search());
    await userEvent.type(search(), "yu go");
    expect(optionNames()).toEqual(["Yu Gothic Médium"]);
  });

  it("says so when no installed font matches", async () => {
    await open();
    await userEvent.type(search(), "zzzz");
    expect(screen.queryAllByRole("option")).toHaveLength(0);
    expect(screen.getByText("Ninguna fuente instalada se llama así")).toBeInTheDocument();
  });

  it("does nothing on Enter when nothing matches", async () => {
    const onPick = await open();
    await userEvent.type(search(), "zzzz{Enter}");
    expect(onPick).not.toHaveBeenCalled();
    expect(screen.getByRole("listbox")).toBeInTheDocument();
  });

  it("picks the family that is clicked, by its English name", async () => {
    const onPick = await open();
    await userEvent.click(screen.getByRole("option", { name: "Yu Gothic Médium" }));
    expect(onPick).toHaveBeenCalledWith("Yu Gothic");
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  it("picks the highlighted one with the keyboard", async () => {
    const onPick = await open();
    await userEvent.keyboard("{ArrowDown}{ArrowDown}{Enter}");
    expect(onPick).toHaveBeenCalledWith("Courier New");
  });

  it("picks the first match after typing", async () => {
    const onPick = await open();
    await userEvent.type(search(), "cour{Enter}");
    expect(onPick).toHaveBeenCalledWith("Courier New");
  });

  it("moves with the arrows, Home and End without leaving either end", async () => {
    await open();
    const lit = () => search().getAttribute("aria-activedescendant");
    const id = (name: string) => screen.getByRole("option", { name }).id;
    expect(lit()).toBe(id(SYSTEM));
    await userEvent.keyboard("{ArrowUp}");
    expect(lit()).toBe(id(SYSTEM));
    await userEvent.keyboard("{End}");
    expect(lit()).toBe(id('Odd"Name'));
    await userEvent.keyboard("{ArrowDown}");
    expect(lit()).toBe(id('Odd"Name'));
    await userEvent.keyboard("{ArrowUp}");
    expect(lit()).toBe(id("Yu Gothic Médium"));
    await userEvent.keyboard("{Home}");
    expect(lit()).toBe(id(SYSTEM));
  });

  it("starts the highlight on the current family", async () => {
    await open("Arial");
    expect(search().getAttribute("aria-activedescendant")).toBe(
      screen.getByRole("option", { name: "Arial" }).id,
    );
  });

  it("sends null when System is picked over a family", async () => {
    const onPick = await open("Arial");
    await userEvent.click(screen.getByRole("option", { name: SYSTEM }));
    expect(onPick).toHaveBeenCalledWith(null);
  });

  it("sends nothing when the current one is picked again", async () => {
    const onPick = await open("Arial");
    await userEvent.click(screen.getByRole("option", { name: "Arial" }));
    expect(onPick).not.toHaveBeenCalled();
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  it("closes on Escape without a change and gives the focus back", async () => {
    const onPick = await open();
    await userEvent.type(search(), "ari{Escape}");
    expect(onPick).not.toHaveBeenCalled();
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(opener(SYSTEM)).toHaveFocus();
  });

  it("forgets the search when it is opened again", async () => {
    await open();
    await userEvent.type(search(), "ari{Escape}");
    await userEvent.click(opener(SYSTEM));
    expect(search()).toHaveValue("");
    expect(screen.getAllByRole("option")).toHaveLength(5);
  });

  it("closes when the button is pressed again", async () => {
    await open();
    await userEvent.click(opener(SYSTEM));
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  it("closes when the focus leaves the search", async () => {
    await open();
    await userEvent.click(document.body);
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  it("closes when the page scrolls away, but not when its own list does", async () => {
    await open();
    screen.getByRole("listbox").dispatchEvent(new Event("scroll"));
    expect(screen.getByRole("listbox")).toBeInTheDocument();
    window.dispatchEvent(new Event("scroll"));
    await vi.waitFor(() => expect(screen.queryByRole("listbox")).toBeNull());
  });

  it("sends null when System is picked while the saved font is not installed", async () => {
    const onPick = mount("Gone Sans");
    await userEvent.click(opener(SYSTEM));
    await userEvent.click(screen.getByRole("option", { name: SYSTEM }));
    expect(onPick).toHaveBeenCalledWith(null);
  });

  it("closes when the window is resized", async () => {
    await open();
    window.dispatchEvent(new Event("resize"));
    await vi.waitFor(() => expect(screen.queryByRole("listbox")).toBeNull());
  });

  it("hangs below the button when there is room", async () => {
    buttonAt(100, 130);
    mount();
    await userEvent.click(opener(SYSTEM));
    expect(drop().style.top).toBe("134px");
    expect(drop().style.bottom).toBe("");
    expect(drop().style.right).toBe(`${window.innerWidth - 700}px`);
  });

  it("opens upward when the button is at the bottom of the window", async () => {
    const low = window.innerHeight - 40;
    buttonAt(low, low + 30);
    mount();
    await userEvent.click(opener(SYSTEM));
    expect(drop().style.bottom).toBe("44px");
    expect(drop().style.top).toBe("");
  });

  it("takes the roomier side and fits it when the list fits neither", async () => {
    vi.spyOn(window, "innerHeight", "get").mockReturnValue(441);
    buttonAt(220, 250);
    mount();
    await userEvent.click(opener(SYSTEM));
    expect(drop().style.bottom).toBe("225px");
    expect(drop().style.maxHeight).toBe("212px");
  });
});
