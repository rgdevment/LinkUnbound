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
};

const EDGE = {
  ...CHROME,
  id: "microsoft-edge",
  name: "Microsoft Edge",
  profiles: 1,
  hidden: true,
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
    answers([CHROME, EDGE, MINE]);
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
    await userEvent.type(screen.getByLabelText("Argumentos adicionales"), "--incognito --new");
    await userEvent.click(screen.getByRole("button", { name: "Añadir" }));
    expect(invoke).toHaveBeenCalledWith("browsers_add", {
      name: "Chrome limpio",
      exe: "C:/x/chrome.exe",
      args: ["--incognito", "--new"],
    });
  });

  it("keeps the form open and says why when the path is wrong", async () => {
    answers([CHROME], { browsers_add: () => Promise.reject("there is no program at that path") });
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir un navegador" }));
    await userEvent.type(screen.getByLabelText("Nombre"), "Roto");
    await userEvent.type(screen.getByLabelText("Ruta del ejecutable"), "C:/nope.exe");
    await userEvent.click(screen.getByRole("button", { name: "Añadir" }));
    expect(await screen.findByText(/no program at that path/)).toBeInTheDocument();
    expect(screen.getByLabelText("Nombre")).toBeInTheDocument();
  });
});
