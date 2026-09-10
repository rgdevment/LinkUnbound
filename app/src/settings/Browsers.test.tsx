import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Browsers from "./Browsers";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const CHROME = {
  id: "google-chrome",
  name: "Google Chrome",
  exe: "C:/Program Files/Google/Chrome/chrome.exe",
  profiles: 2,
  private: true,
  custom: false,
  hidden: false,
  icon: "data:image/png;base64,iVBORw0KGgo=",
  args: [],
  private_flag: "--incognito",
  icon_path: null,
};

const EDGE = {
  ...CHROME,
  id: "microsoft-edge",
  name: "Microsoft Edge",
  profiles: 1,
  hidden: true,
};

const ODD = {
  ...CHROME,
  id: "odd-browser",
  name: "Navegador raro",
  profiles: 0,
  private: false,
  private_flag: null,
  hidden: false,
};

const MINE = {
  id: "custom-1",
  name: "Chrome sin extensiones",
  exe: "C:/Program Files/Google/Chrome/chrome.exe",
  profiles: 0,
  private: false,
  custom: true,
  hidden: false,
  icon: null,
  args: ["--disable-extensions"],
  private_flag: null,
  icon_path: null,
};

function answers(list: unknown[], overrides: Record<string, () => Promise<unknown>> = {}) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd in overrides) return overrides[cmd]();
    return Promise.resolve(list);
  });
}

describe("browsers", () => {
  beforeEach(() => {
    invoke.mockReset();
    answers([CHROME, EDGE, ODD, MINE]);
  });

  it("tells apart what Windows reports from what the user added", async () => {
    render(<Browsers />);
    expect(await screen.findByText("Detectados en el equipo")).toBeInTheDocument();
    expect(screen.getByText("Añadidos por ti")).toBeInTheDocument();
    expect(screen.getByText("Chrome sin extensiones")).toBeInTheDocument();
  });

  it("says how many profiles a browser has and whether it opens private windows", async () => {
    render(<Browsers />);
    expect(await screen.findByText("2 perfiles · admite ventana privada")).toBeInTheDocument();
    expect(
      screen.getByText("1 perfil · admite ventana privada · oculto del selector"),
    ).toBeInTheDocument();
  });

  it("hides a browser from the picker without forgetting it", async () => {
    render(<Browsers />);
    await userEvent.click(
      await screen.findByRole("switch", { name: "Mostrar Google Chrome en el selector" }),
    );
    expect(invoke).toHaveBeenCalledWith("browsers_set_hidden", {
      id: "google-chrome",
      hidden: true,
    });
  });

  it("shows a hidden browser as still listed, only turned off", async () => {
    render(<Browsers />);
    const toggle = await screen.findByRole("switch", {
      name: "Mostrar Microsoft Edge en el selector",
    });
    expect(toggle).toHaveAttribute("aria-checked", "false");
  });

  /// A detected browser has no delete button at all: removing it would only
  /// last until the next scan.
  it("only offers to delete the browsers the user added", async () => {
    render(<Browsers />);
    await screen.findByText("Google Chrome");
    expect(screen.queryByRole("button", { name: "Eliminar Google Chrome" })).toBeNull();
    expect(
      screen.getByRole("button", { name: "Eliminar Chrome sin extensiones" }),
    ).toBeInTheDocument();
  });

  it("sends a new browser with its arguments split apart", async () => {
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir un navegador" }));
    await userEvent.type(screen.getByLabelText("Nombre"), "Chrome limpio");
    await userEvent.type(screen.getByLabelText("Ruta del ejecutable"), "C:/x/chrome.exe");
    await userEvent.type(screen.getByLabelText("Argumentos adicionales"), "--new --foo");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(invoke).toHaveBeenCalledWith("browsers_add", {
      edit: {
        name: "Chrome limpio",
        exe: "C:/x/chrome.exe",
        args: ["--new", "--foo"],
        private_flag: null,
        icon_path: null,
      },
    });
  });

  /// Without this argument the browser can never open a private window, which
  /// is why the form asks for it instead of guessing.
  it("carries the private argument so a custom browser can open incognito", async () => {
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir un navegador" }));
    await userEvent.type(screen.getByLabelText("Nombre"), "Brave");
    await userEvent.type(screen.getByLabelText("Ruta del ejecutable"), "C:/b.exe");
    await userEvent.type(screen.getByLabelText("Argumento de ventana privada"), "--incognito");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(invoke).toHaveBeenCalledWith("browsers_add", {
      edit: expect.objectContaining({ private_flag: "--incognito" }),
    });
  });

  it("says which browsers cannot open a private window at all", async () => {
    render(<Browsers />);
    expect(await screen.findByText("Sin perfiles · sin ventana privada")).toBeInTheDocument();
  });

  it("opens an editable form filled with what the browser already had", async () => {
    render(<Browsers />);
    await userEvent.click(
      await screen.findByRole("button", { name: "Editar Chrome sin extensiones" }),
    );
    expect(screen.getByLabelText("Nombre")).toHaveValue("Chrome sin extensiones");
    expect(screen.getByLabelText("Argumentos adicionales")).toHaveValue("--disable-extensions");
  });

  it("saves an edit against the browser being edited", async () => {
    render(<Browsers />);
    await userEvent.click(
      await screen.findByRole("button", { name: "Editar Chrome sin extensiones" }),
    );
    await userEvent.clear(screen.getByLabelText("Nombre"));
    await userEvent.type(screen.getByLabelText("Nombre"), "Otro nombre");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(invoke).toHaveBeenCalledWith("browsers_update", {
      id: "custom-1",
      edit: expect.objectContaining({ name: "Otro nombre" }),
    });
  });

  /// Duplicating a detected browser is how one binary becomes two entries with
  /// different arguments, so it cannot be limited to the custom ones.
  it("can duplicate a detected browser, not only a custom one", async () => {
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Duplicar Google Chrome" }));
    expect(invoke).toHaveBeenCalledWith("browsers_duplicate", { id: "google-chrome" });
  });

  it("sends the whole order when a browser moves, since that is the picker order", async () => {
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Bajar Google Chrome" }));
    expect(invoke).toHaveBeenCalledWith("browsers_reorder", {
      ids: ["microsoft-edge", "google-chrome", "odd-browser", "custom-1"],
    });
  });

  it("keeps the form open and says why when the path is wrong", async () => {
    answers([CHROME], { browsers_add: () => Promise.reject("no hay ningún programa en esa ruta") });
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir un navegador" }));
    await userEvent.type(screen.getByLabelText("Nombre"), "Roto");
    await userEvent.type(screen.getByLabelText("Ruta del ejecutable"), "C:/nope.exe");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(await screen.findByText(/ningún programa en esa ruta/)).toBeInTheDocument();
    expect(screen.getByLabelText("Nombre")).toBeInTheDocument();
  });
});
