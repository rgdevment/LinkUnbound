import { render, screen, within } from "@testing-library/react";
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
  args: "",
  private_flag: "--incognito",
  icon_path: null,
  profile_names: [
    ["Default", "Personal"],
    ["Profile 1", "Trabajo"],
  ] as [string, string][],
  keys: { "Profile 1": "W" } as Record<string, string>,
  shown_keys: { Default: "1", "Profile 1": "W" } as Record<string, string>,
};

const EDGE = {
  ...CHROME,
  id: "microsoft-edge",
  name: "Microsoft Edge",
  profiles: 1,
  hidden: true,
  profile_names: [["Default", "Personal"]] as [string, string][],
  keys: {},
  shown_keys: {},
};

const ODD = {
  ...CHROME,
  id: "odd-browser",
  name: "Navegador raro",
  profiles: 0,
  private: false,
  private_flag: null,
  hidden: false,
  profile_names: [],
  keys: {},
  shown_keys: { "": "2" },
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
  args: "--disable-extensions",
  private_flag: null,
  icon_path: null,
  profile_names: [] as [string, string][],
  keys: {} as Record<string, string>,
  shown_keys: { "": "3" } as Record<string, string>,
};

function answers(list: unknown[], overrides: Record<string, () => Promise<unknown>> = {}) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd in overrides) return overrides[cmd]();
    return Promise.resolve(list);
  });
}

describe("browsers", () => {
  it("asks before removing a browser, and says what is lost", async () => {
    render(<Browsers />);
    await userEvent.click(
      await screen.findByRole("button", { name: "Eliminar Chrome sin extensiones" }),
    );
    expect(invoke).not.toHaveBeenCalledWith("browsers_remove", expect.anything());
    expect(await screen.findByText(/sin destino/)).toBeInTheDocument();

    const ask = screen.getByRole("alertdialog", { name: "Eliminar Chrome sin extensiones" });
    await userEvent.click(within(ask).getByRole("button", { name: "Eliminar" }));
    expect(invoke).toHaveBeenCalledWith("browsers_remove", { id: MINE.id });
  });

  /// The saved order is one list and the screen shows two: stepping through the
  /// whole list moved something in the other section, invisibly.
  it("moves a browser past the one above it on screen, not in the file", async () => {
    answers([MINE, CHROME, EDGE, ODD]);
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Bajar Google Chrome" }));
    expect(invoke).toHaveBeenCalledWith("browsers_reorder", {
      ids: [MINE.id, EDGE.id, CHROME.id, ODD.id],
    });
  });

  it("does not offer to move the first of a section above a browser in the other", async () => {
    answers([MINE, CHROME, EDGE, ODD]);
    render(<Browsers />);
    expect(await screen.findByRole("button", { name: "Subir Google Chrome" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Subir Chrome sin extensiones" })).toBeDisabled();
  });

  /// Only the path is off limits: a rescan would undo anything typed over it.
  it("edits a detected browser, with its path locked and its private flag out of reach", async () => {
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Editar Google Chrome" }));
    expect(await screen.findByLabelText("Ruta del ejecutable")).toHaveAttribute("readonly");
    expect(screen.queryByLabelText("Argumento de ventana privada")).toBeNull();

    const name = screen.getByLabelText("Nombre");
    await userEvent.clear(name);
    await userEvent.type(name, "Chrome del trabajo");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));

    expect(invoke).toHaveBeenCalledWith(
      "browsers_update",
      expect.objectContaining({ id: "google-chrome" }),
    );
  });

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

  /// The line travels whole. Splitting it here is what broke a profile whose name has a space
  /// in it — the most ordinary argument Chrome and Firefox take — so what an argument is gets
  /// decided in one place, and that place has tests of its own.
  it("sends a new browser with the arguments exactly as they were typed", async () => {
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir un navegador" }));
    await userEvent.type(screen.getByLabelText("Nombre"), "Chrome limpio");
    await userEvent.type(screen.getByLabelText("Ruta del ejecutable"), "C:/x/chrome.exe");
    await userEvent.type(
      screen.getByLabelText("Argumentos adicionales"),
      '--profile-directory="Profile 1"',
    );
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(invoke).toHaveBeenCalledWith("browsers_add", {
      edit: {
        name: "Chrome limpio",
        exe: "C:/x/chrome.exe",
        args: '--profile-directory="Profile 1"',
        private_flag: null,
        icon_path: null,
        keys: {},
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

  /// A key another row holds is named before saving, and the form will not save it.
  it("warns when the key is already another browser's", async () => {
    render(<Browsers />);
    await userEvent.click(
      await screen.findByRole("button", { name: "Editar Chrome sin extensiones" }),
    );
    await userEvent.type(screen.getByLabelText("Tecla en el selector"), "w");
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Esa tecla ya la usa Google Chrome · Trabajo",
    );
    expect(screen.getByRole("button", { name: "Guardar" })).toBeDisabled();

    await userEvent.type(screen.getByLabelText("Tecla en el selector"), "p");
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(invoke).toHaveBeenCalledWith("browsers_update", {
      id: "custom-1",
      edit: expect.objectContaining({ keys: { "": "P" } }),
    });
  });

  /// Each profile is a row of its own in the picker, so each gets its own key.
  it("offers a key for every profile and keeps the one already given", async () => {
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Editar Google Chrome" }));
    expect(screen.getByLabelText("Tecla de Trabajo")).toHaveValue("W");
    expect(screen.getByLabelText("Tecla de Personal")).toHaveValue("");
    expect(screen.getByLabelText("Tecla de Personal")).toHaveAttribute("placeholder", "1");
    await userEvent.type(screen.getByLabelText("Tecla de Personal"), "w");
    for (const said of screen.getAllByRole("alert")) {
      expect(said).toHaveTextContent("Dos perfiles no pueden compartir tecla");
    }
    await userEvent.type(screen.getByLabelText("Tecla de Personal"), "1");
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  /// The advanced options carry examples, since few people know these switches exist.
  it("shows how to make a profile or a separate browser", async () => {
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir un navegador" }));
    await userEvent.click(
      screen.getByRole("button", { name: "Ejemplos: perfiles y navegadores personalizados" }),
    );
    expect(screen.getByText('--profile-directory="Profile 1"')).toBeVisible();
    expect(screen.getByText('--user-data-dir="C:/Navegadores/Trabajo"')).toBeVisible();
  });

  /// The box holds one key, so the new one is whatever was typed, wherever the caret was; and a
  /// hidden browser is not in the picker, so its key is free.
  it("takes the key just typed and ignores the ones hidden browsers kept", async () => {
    answers([CHROME, { ...EDGE, keys: { Default: "E" } }, ODD, MINE]);
    render(<Browsers />);
    await userEvent.click(
      await screen.findByRole("button", { name: "Editar Chrome sin extensiones" }),
    );
    const box = screen.getByLabelText("Tecla en el selector");
    await userEvent.type(box, "e");
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    await userEvent.type(box, "k", { initialSelectionStart: 0, initialSelectionEnd: 0 });
    expect(box).toHaveValue("K");
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
    answers([CHROME], { browsers_add: () => Promise.reject("browserNoProgram") });
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Añadir un navegador" }));
    await userEvent.type(screen.getByLabelText("Nombre"), "Roto");
    await userEvent.type(screen.getByLabelText("Ruta del ejecutable"), "C:/nope.exe");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(
      await screen.findByText("No hay ningún programa en esa ruta"),
      "the key has to arrive as a sentence, not as the key",
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Nombre")).toBeInTheDocument();
  });

  /// The confirmation promises the hand-added browsers survive, and that was
  /// asserted as text and never as the command that runs.
  it("looks for browsers without wiping anything", async () => {
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Buscar" }));
    const ask = await screen.findByRole("alertdialog", { name: "Volver a buscar navegadores" });
    await userEvent.click(within(ask).getByRole("button", { name: "Buscar" }));

    expect(invoke).toHaveBeenCalledWith("maintenance_rescan");
    expect(invoke).not.toHaveBeenCalledWith("maintenance_reset", expect.anything());
  });

  it("explains that a rescan keeps what the user set and shows what was hidden", async () => {
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Buscar" }));
    expect(
      await screen.findByText(/Se conservan los nombres, argumentos, iconos/),
    ).toBeInTheDocument();
    expect(screen.getByText(/añadiste a mano/)).toBeInTheDocument();
  });

  /// «Done» alone reads the same whether a browser appeared or nothing changed.
  it("counts what the rescan found and what it lost, and shows the new list", async () => {
    answers([CHROME, EDGE, ODD, MINE], {
      maintenance_rescan: () => Promise.resolve({ added: 2, removed: 1 }),
    });
    render(<Browsers />);
    await userEvent.click(await screen.findByRole("button", { name: "Buscar" }));
    const ask = await screen.findByRole("alertdialog", { name: "Volver a buscar navegadores" });
    await userEvent.click(within(ask).getByRole("button", { name: "Buscar" }));

    expect(await screen.findByText(/2 nuevos, 1 que ya no están/)).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("browsers_list", {});
  });
});
