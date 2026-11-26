import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { follow } from "../theme";
import Appearance from "./Appearance";
import type { Look } from "./look";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const LOOK: Look = {
  theme: "system",
  high_contrast: false,
  text_size: "normal",
  picker_style: "bear",
  density: "normal",
  picker_position: "pointer",
  font: null,
};

const FONTS = {
  system: "Segoe UI",
  families: [
    { name: "Arial", label: "Arial" },
    { name: "Courier New", label: "Courier New" },
  ],
};

const REFUSAL = "the file could not be written";

function answers(start: Look = LOOK, fail?: string) {
  let held = start;
  invoke.mockImplementation((cmd: string, args?: { appearance?: Look }) => {
    if (cmd === "appearance_get") return Promise.resolve(held);
    if (cmd === "appearance_set") {
      if (fail) return Promise.reject(fail);
      held = args?.appearance ?? held;
      return Promise.resolve(held);
    }
    if (cmd === "appearance_reset") {
      held = LOOK;
      return Promise.resolve(held);
    }
    if (cmd === "prefs_get") return Promise.resolve({ prefs: { ...held } });
    if (cmd === "fonts_list") return Promise.resolve(FONTS);
    return Promise.resolve(null);
  });
}

async function mount() {
  const view = render(<Appearance />);
  await screen.findByRole("group", { name: "Tema" });
  return view;
}

const previews = () => invoke.mock.calls.filter(([cmd]) => cmd === "picker_preview").length;

function group(name: string) {
  return within(screen.getByRole("group", { name }));
}

function miniature(container: HTMLElement) {
  return container.querySelector<HTMLElement>(".look-picker") as HTMLElement;
}

describe("appearance settings", () => {
  beforeEach(() => {
    invoke.mockReset();
    answers();
    document.documentElement.className = "";
  });

  it("shows what is in force on every control", async () => {
    answers({
      theme: "dark",
      high_contrast: true,
      text_size: "large",
      picker_style: "sheet",
      density: "comfortable",
      picker_position: "center",
      font: null,
    });
    await mount();
    expect(group("Tema").getByRole("radio", { name: "Oscuro" })).toBeChecked();
    expect(group("Tema").getByRole("radio", { name: "Automático" })).not.toBeChecked();
    expect(screen.getByRole("switch", { name: "Contraste alto" })).toBeChecked();
    expect(group("Tamaño del texto").getByRole("radio", { name: "Grande" })).toBeChecked();
    expect(group("Estilo").getByRole("radio", { name: "Mosaico" })).toBeChecked();
    expect(group("Densidad").getByRole("radio", { name: "Amplia" })).toBeChecked();
    expect(group("Dónde aparece").getByRole("radio", { name: "Al centro" })).toBeChecked();
  });

  it("saves the theme as a choice of its own, not as whatever the system says", async () => {
    await mount();
    await userEvent.click(group("Tema").getByRole("radio", { name: "Oscuro" }));
    expect(invoke).toHaveBeenCalledWith("appearance_set", {
      appearance: { ...LOOK, theme: "dark" },
    });
  });

  it("repaints the window as soon as the theme is chosen", async () => {
    vi.stubGlobal("matchMedia", () => ({ matches: false, addEventListener() {} }));
    follow();
    await mount();
    await userEvent.click(group("Tema").getByRole("radio", { name: "Oscuro" }));
    await waitFor(() => expect(document.documentElement).toHaveClass("dark"));
  });

  it("sends high contrast on and off", async () => {
    await mount();
    const contrast = screen.getByRole("switch", { name: "Contraste alto" });
    await userEvent.click(contrast);
    expect(invoke).toHaveBeenCalledWith("appearance_set", {
      appearance: { ...LOOK, high_contrast: true },
    });
    await waitFor(() => expect(contrast).toBeChecked());
    await userEvent.click(contrast);
    expect(invoke).toHaveBeenCalledWith("appearance_set", {
      appearance: { ...LOOK, high_contrast: false },
    });
  });

  it("sends the text size that was chosen", async () => {
    await mount();
    await userEvent.click(group("Tamaño del texto").getByRole("radio", { name: "Pequeño" }));
    expect(invoke).toHaveBeenCalledWith("appearance_set", {
      appearance: { ...LOOK, text_size: "small" },
    });
  });

  it("keeps the rest of the look when one thing changes", async () => {
    const look: Look = { ...LOOK, theme: "light", high_contrast: true, density: "compact" };
    answers(look);
    await mount();
    await userEvent.click(group("Tamaño del texto").getByRole("radio", { name: "Grande" }));
    expect(invoke).toHaveBeenCalledWith("appearance_set", {
      appearance: { ...look, text_size: "large" },
    });
  });

  it("offers the four looks and sends the one chosen", async () => {
    await mount();
    for (const name of ["LinkBear", "Clásico", "Mosaico", "Compacto"]) {
      expect(group("Estilo").getByRole("radio", { name })).toBeInTheDocument();
    }
    await userEvent.click(group("Estilo").getByRole("radio", { name: "Mosaico" }));
    expect(invoke).toHaveBeenCalledWith("appearance_set", {
      appearance: { ...LOOK, picker_style: "sheet" },
    });
  });

  it("opens the picker by the pointer until the center is chosen", async () => {
    await mount();
    expect(group("Dónde aparece").getByRole("radio", { name: "Junto al puntero" })).toBeChecked();
    await userEvent.click(group("Dónde aparece").getByRole("radio", { name: "Al centro" }));
    expect(invoke).toHaveBeenCalledWith("appearance_set", {
      appearance: { ...LOOK, picker_position: "center" },
    });
  });

  it("asks the shell to show the picker over a sample link", async () => {
    await mount();
    await userEvent.click(screen.getByRole("button", { name: "Vista previa" }));
    expect(invoke).toHaveBeenCalledWith("picker_preview");
  });

  /// Once a preview is up, choosing another look redraws it rather than leaving the old one open.
  it("redraws the real preview only after it was asked for", async () => {
    await mount();
    const mosaic = group("Estilo").getByRole("radio", { name: "Mosaico" });
    await userEvent.click(mosaic);
    await waitFor(() => expect(mosaic).toBeChecked());
    expect(previews()).toBe(0);

    await userEvent.click(screen.getByRole("button", { name: "Vista previa" }));
    expect(previews()).toBe(1);
    await userEvent.click(group("Estilo").getByRole("radio", { name: "LinkBear" }));
    await waitFor(() => expect(previews()).toBe(2));
    await userEvent.click(group("Tamaño del texto").getByRole("radio", { name: "Grande" }));
    await waitFor(() => expect(previews()).toBe(3));
  });

  it("does not redraw the preview when the change was refused", async () => {
    answers(LOOK, REFUSAL);
    await mount();
    await userEvent.click(screen.getByRole("button", { name: "Vista previa" }));
    await userEvent.click(group("Tamaño del texto").getByRole("radio", { name: "Grande" }));
    await screen.findByText(/could not be written/);
    expect(previews()).toBe(1);
  });

  it("surfaces the backend refusal rather than looking like it worked", async () => {
    answers(LOOK, REFUSAL);
    await mount();
    await userEvent.click(group("Tema").getByRole("radio", { name: "Claro" }));
    expect(await screen.findByText(/could not be written/)).toBeInTheDocument();
  });

  describe("when a change is refused", () => {
    it("puts the pressed control back on its old value and keeps the rest", async () => {
      const held: Look = { ...LOOK, theme: "light", text_size: "small" };
      answers(held, REFUSAL);
      await mount();
      await userEvent.click(group("Tamaño del texto").getByRole("radio", { name: "Grande" }));
      await screen.findByText(/could not be written/);

      expect(group("Tamaño del texto").getByRole("radio", { name: "Pequeño" })).toBeChecked();
      expect(group("Tamaño del texto").getByRole("radio", { name: "Grande" })).not.toBeChecked();
      expect(group("Tema").getByRole("radio", { name: "Claro" })).toBeChecked();
    });

    it("puts the switch back off", async () => {
      answers(LOOK, REFUSAL);
      await mount();
      const contrast = screen.getByRole("switch", { name: "Contraste alto" });
      await userEvent.click(contrast);
      await screen.findByText(/could not be written/);
      expect(contrast).not.toBeChecked();
    });

    it("draws the miniature as it was", async () => {
      answers(LOOK, REFUSAL);
      const { container } = await mount();
      await userEvent.click(group("Estilo").getByRole("radio", { name: "Compacto" }));
      await userEvent.click(screen.getByRole("switch", { name: "Contraste alto" }));
      await userEvent.click(group("Tamaño del texto").getByRole("radio", { name: "Grande" }));
      await screen.findByText(/could not be written/);

      const picker = miniature(container);
      expect(picker).toHaveAttribute("data-shape", "cards");
      expect(picker).not.toHaveClass("strong");
      expect(picker.style.getPropertyValue("--zoom")).toBe("1");
    });

    it("lets go of the refusal once a later change is accepted", async () => {
      answers(LOOK, REFUSAL);
      await mount();
      await userEvent.click(group("Tema").getByRole("radio", { name: "Oscuro" }));
      await screen.findByText(/could not be written/);

      answers(LOOK);
      await userEvent.click(group("Tema").getByRole("radio", { name: "Oscuro" }));
      await waitFor(() => expect(screen.queryByText(/could not be written/)).toBeNull());
      expect(group("Tema").getByRole("radio", { name: "Oscuro" })).toBeChecked();
    });
  });

  it("shows nothing until the look has been read", () => {
    invoke.mockReturnValue(new Promise(() => {}));
    const { container } = render(<Appearance />);
    expect(container).toBeEmptyDOMElement();
  });

  it("survives a backend that cannot give the look", async () => {
    invoke.mockRejectedValue("no bridge");
    const { container } = render(<Appearance />);
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(container).toBeEmptyDOMElement();
  });

  describe("density", () => {
    it.each([
      ["classic", "compact", "Cada fila mide 36 px"],
      ["classic", "normal", "Cada fila mide 45 px"],
      ["classic", "comfortable", "Cada fila mide 54 px"],
      ["bear", "compact", "Cada tarjeta mide 44 px"],
      ["bear", "normal", "Cada tarjeta mide 52 px"],
      ["bear", "comfortable", "Cada tarjeta mide 60 px"],
      ["sheet", "compact", "Cada tarjeta mide 44 px"],
      ["sheet", "normal", "Cada tarjeta mide 52 px"],
      ["sheet", "comfortable", "Cada tarjeta mide 60 px"],
    ] as const)("says the size of %s at %s", async (picker_style, density, note) => {
      answers({ ...LOOK, picker_style, density });
      await mount();
      expect(screen.getByText(note)).toBeInTheDocument();
      expect(group("Densidad").getByRole("radio", { name: "Normal" })).toBeEnabled();
    });

    it("is switched off for the compact look, which has no use for it", async () => {
      answers({ ...LOOK, picker_style: "compact" });
      await mount();
      expect(
        screen.getByText("El estilo Compacto muestra solo iconos y no la usa"),
      ).toBeInTheDocument();
      expect(group("Densidad").getByRole("radio", { name: "Normal" })).toBeDisabled();
      expect(group("Densidad").getByRole("radio", { name: "Amplia" })).toBeDisabled();
    });

    it("changes the note as soon as the style does", async () => {
      answers({ ...LOOK, picker_style: "classic" });
      await mount();
      await userEvent.click(group("Estilo").getByRole("radio", { name: "Compacto" }));
      expect(await screen.findByText(/y no la usa/)).toBeInTheDocument();
    });

    it("sends the density that was chosen", async () => {
      await mount();
      await userEvent.click(group("Densidad").getByRole("radio", { name: "Amplia" }));
      expect(invoke).toHaveBeenCalledWith("appearance_set", {
        appearance: { ...LOOK, density: "comfortable" },
      });
    });
  });

  describe("reset", () => {
    it("asks for the defaults and shows what comes back", async () => {
      answers({
        theme: "dark",
        high_contrast: true,
        text_size: "large",
        picker_style: "compact",
        density: "comfortable",
        picker_position: "center",
        font: null,
      });
      await mount();
      await userEvent.click(screen.getByRole("button", { name: "Restablecer la apariencia" }));

      expect(invoke).toHaveBeenCalledWith("appearance_reset");
      await waitFor(() =>
        expect(group("Tema").getByRole("radio", { name: "Automático" })).toBeChecked(),
      );
      expect(screen.getByRole("switch", { name: "Contraste alto" })).not.toBeChecked();
      expect(group("Tamaño del texto").getByRole("radio", { name: "Normal" })).toBeChecked();
      expect(group("Estilo").getByRole("radio", { name: "LinkBear" })).toBeChecked();
      expect(group("Densidad").getByRole("radio", { name: "Normal" })).toBeChecked();
      expect(group("Dónde aparece").getByRole("radio", { name: "Junto al puntero" })).toBeChecked();
    });

    it("keeps what was in force when the reset is refused", async () => {
      const held: Look = { ...LOOK, theme: "dark", high_contrast: true, text_size: "large" };
      invoke.mockImplementation((cmd: string) =>
        cmd === "appearance_reset"
          ? Promise.reject(REFUSAL)
          : Promise.resolve(cmd === "fonts_list" ? FONTS : held),
      );
      const { container } = await mount();
      await userEvent.click(screen.getByRole("button", { name: "Restablecer la apariencia" }));
      await screen.findByText(/could not be written/);

      expect(group("Tema").getByRole("radio", { name: "Oscuro" })).toBeChecked();
      expect(screen.getByRole("switch", { name: "Contraste alto" })).toBeChecked();
      expect(group("Tamaño del texto").getByRole("radio", { name: "Grande" })).toBeChecked();
      expect(miniature(container)).toHaveClass("strong");
    });

    it("surfaces a refusal", async () => {
      invoke.mockImplementation((cmd: string) =>
        cmd === "appearance_reset"
          ? Promise.reject(REFUSAL)
          : Promise.resolve(cmd === "fonts_list" ? FONTS : LOOK),
      );
      await mount();
      await userEvent.click(screen.getByRole("button", { name: "Restablecer la apariencia" }));
      expect(await screen.findByText(/could not be written/)).toBeInTheDocument();
    });
  });

  describe("the miniature", () => {
    it.each([
      ["classic", "list"],
      ["bear", "cards"],
      ["sheet", "cards"],
      ["compact", "icons"],
    ] as const)("draws %s as %s", async (picker_style, shape) => {
      answers({ ...LOOK, picker_style });
      const { container } = await mount();
      expect(miniature(container)).toHaveAttribute("data-shape", shape);
    });

    it("turns strong with high contrast, and only then", async () => {
      const { container } = await mount();
      expect(miniature(container)).not.toHaveClass("strong");
      await userEvent.click(screen.getByRole("switch", { name: "Contraste alto" }));
      await waitFor(() => expect(miniature(container)).toHaveClass("strong"));
    });

    it("follows the size and the density in its measures", async () => {
      answers({ ...LOOK, text_size: "large", density: "comfortable" });
      const { container } = await mount();
      const style = miniature(container).style;
      expect(style.getPropertyValue("--zoom")).toBe("1.13");
      expect(style.getPropertyValue("--row")).toBe("54px");
      expect(style.getPropertyValue("--card")).toBe("60px");
      expect(style.getPropertyValue("--icon")).toBe("28px");
    });

    it("shrinks with a small size and a compact density", async () => {
      answers({ ...LOOK, text_size: "small", density: "compact" });
      const { container } = await mount();
      const style = miniature(container).style;
      expect(style.getPropertyValue("--zoom")).toBe("0.87");
      expect(style.getPropertyValue("--row")).toBe("36px");
      expect(style.getPropertyValue("--card")).toBe("44px");
      expect(style.getPropertyValue("--icon")).toBe("20px");
    });

    it.each(["classic", "bear", "compact"] as const)(
      "names the sample host and browser in the %s shape",
      async (picker_style) => {
        answers({ ...LOOK, picker_style });
        const { container } = await mount();
        expect(miniature(container)).toHaveTextContent("linkunbound.app");
        expect(miniature(container)).toHaveTextContent("Chrome");
      },
    );
  });

  describe("the font", () => {
    const opener = (current: string) => screen.getByRole("button", { name: `Fuente: ${current}` });
    const SYSTEM = "Sistema (Segoe UI) · predeterminada";

    it("shows System as the font in force by default", async () => {
      await mount();
      expect(opener(SYSTEM)).toBeInTheDocument();
      expect(screen.getByText("Cualquiera instalada en el equipo")).toBeInTheDocument();
    });

    it("shows the family that is in force", async () => {
      answers({ ...LOOK, font: "Courier New" });
      await mount();
      expect(opener("Courier New")).toBeInTheDocument();
    });

    it("sends the English name of the family that was picked", async () => {
      await mount();
      await userEvent.click(opener(SYSTEM));
      await userEvent.click(screen.getByRole("option", { name: "Courier New" }));
      expect(invoke).toHaveBeenCalledWith("appearance_set", {
        appearance: { ...LOOK, font: "Courier New" },
      });
      await waitFor(() => expect(opener("Courier New")).toBeInTheDocument());
    });

    it("sends null when System is picked again", async () => {
      answers({ ...LOOK, font: "Arial" });
      await mount();
      await userEvent.click(opener("Arial"));
      await userEvent.click(screen.getByRole("option", { name: SYSTEM }));
      expect(invoke).toHaveBeenCalledWith("appearance_set", {
        appearance: { ...LOOK, font: null },
      });
    });

    it("sends nothing when the list is closed with Escape", async () => {
      await mount();
      await userEvent.click(opener(SYSTEM));
      await userEvent.keyboard("{Escape}");
      expect(invoke.mock.calls.filter(([cmd]) => cmd === "appearance_set")).toHaveLength(0);
    });

    it("redraws an open preview with the new font", async () => {
      await mount();
      await userEvent.click(screen.getByRole("button", { name: "Vista previa" }));
      await userEvent.click(opener(SYSTEM));
      await userEvent.click(screen.getByRole("option", { name: "Arial" }));
      await waitFor(() => expect(previews()).toBe(2));
    });

    it("puts the old font back when the change is refused", async () => {
      answers(LOOK, REFUSAL);
      await mount();
      await userEvent.click(opener(SYSTEM));
      await userEvent.click(screen.getByRole("option", { name: "Arial" }));
      await screen.findByText(/could not be written/);
      expect(opener(SYSTEM)).toBeInTheDocument();
    });

    it("brings System back on reset", async () => {
      answers({ ...LOOK, font: "Courier New" });
      await mount();
      await userEvent.click(screen.getByRole("button", { name: "Restablecer la apariencia" }));
      await waitFor(() => expect(opener(SYSTEM)).toBeInTheDocument());
    });

    it("keeps the button disabled when the fonts cannot be listed", async () => {
      invoke.mockImplementation((cmd: string) =>
        cmd === "fonts_list" ? Promise.reject("no fonts") : Promise.resolve(LOOK),
      );
      await mount();
      expect(screen.getByRole("button", { name: /^Fuente/ })).toBeDisabled();
    });

    it("reads the fonts again when the window comes back to the front", async () => {
      await mount();
      const listed = () => invoke.mock.calls.filter(([cmd]) => cmd === "fonts_list").length;
      expect(listed()).toBe(1);
      window.dispatchEvent(new Event("focus"));
      await waitFor(() => expect(listed()).toBe(2));
    });

    it("writes the miniature in the chosen font, and only then", async () => {
      const { container } = await mount();
      expect(miniature(container).style.getPropertyValue("--look-font")).toBe("");
      await userEvent.click(opener(SYSTEM));
      await userEvent.click(screen.getByRole("option", { name: "Courier New" }));
      await waitFor(() =>
        expect(miniature(container).style.getPropertyValue("--look-font")).toBe('"Courier New"'),
      );
    });
  });
});
