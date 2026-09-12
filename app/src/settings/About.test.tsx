import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import About from "./About";
import type { Ready, Underway } from "./update";

const opened: string[] = [];
const invoke = vi.fn();

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: (url: string) => {
    opened.push(url);
    return Promise.resolve();
  },
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const BUILD = {
  version: "2.0.0",
  license: "GPL-3.0-only",
  repository: "https://github.com/rgdevment/LinkUnbound",
  candidates: false,
  candidatesApply: true,
};

const NEWER: Ready = {
  version: "2.1.0",
  route: "download",
  package: null,
  installs: true,
};

/// The Store can install what the Store itself offered, so this route installs too.
const FROM_STORE: Ready = { ...NEWER, route: "store" };

const settled = vi.fn();

/// Stands in for the window that owns the offer, so a look really does move what the screen
/// reads instead of the test asserting against a copy only it can see.
function Host({ from, step }: { from: Ready | null; step: Underway | null }) {
  const [ready, setReady] = useState(from);
  return (
    <About
      ready={ready}
      step={step}
      onSettled={settled}
      onLook={(nowPlease) =>
        invoke("update_ready", { nowPlease }).then((one: unknown) => {
          setReady((one ?? null) as Ready | null);
          return one;
        })
      }
    />
  );
}

function show(ready: Ready | null = null, step: Underway | null = null) {
  render(<Host from={ready} step={step} />);
}

function answers(overrides: Record<string, () => Promise<unknown>> = {}) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd in overrides) return overrides[cmd]();
    if (cmd === "about") return Promise.resolve(BUILD);
    return Promise.resolve(null);
  });
}

describe("about", () => {
  beforeEach(() => {
    opened.length = 0;
    invoke.mockReset();
    settled.mockReset();
    answers();
  });

  it("names the licence, the coffee and the sibling tool", async () => {
    show();
    expect(screen.getByText("Licencia GPL-3.0")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Invítame un café" })).toBeInTheDocument();
    expect(screen.getByText("CopyPaste")).toBeInTheDocument();
  });

  it("reads the version from the build instead of a number typed into the screen", async () => {
    show();
    expect(await screen.findByText(/2\.0\.0 · GPL-3\.0-only/)).toBeInTheDocument();
  });

  /// A link that navigates the webview would trap the user in the settings window with no way
  /// back, and one that reaches nothing at all is what a plain anchor does here.
  it("hands every link to the system browser", async () => {
    show();
    for (const open of screen.getAllByRole("button", { name: "Abrir" })) {
      await userEvent.click(open);
    }

    expect(opened).toEqual([
      "https://github.com/rgdevment/LinkUnbound",
      "https://github.com/rgdevment/LinkUnbound/blob/main/LICENSE",
      "https://github.com/rgdevment/LinkUnbound/issues",
      "https://buymeacoffee.com/rgdevment",
      "https://github.com/rgdevment/CopyPaste",
    ]);
  });

  it("says the copy is current when nothing newer was found", async () => {
    show();
    expect(screen.getByText("Estás en la última versión")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Actualizar" })).toBeNull();
  });

  it("looks again when asked, and says so while it does", async () => {
    let answer: (one: Ready) => void = () => {};
    answers({ update_ready: () => new Promise<Ready>((give) => (answer = give)) });
    show();

    await userEvent.click(screen.getByRole("button", { name: "Buscar ahora" }));
    expect(invoke).toHaveBeenCalledWith("update_ready", { nowPlease: true });
    expect(screen.getByText("Buscando…")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Buscar ahora" })).toBeDisabled();

    answer(NEWER);
    expect(await screen.findByText("Versión 2.1.0 disponible")).toBeInTheDocument();
  });

  it("installs the version it showed, and only once", async () => {
    show(NEWER);
    const update = screen.getByRole("button", { name: "Actualizar" });
    await userEvent.click(update);
    await userEvent.click(update);

    expect(invoke.mock.calls.filter(([cmd]) => cmd === "update_install")).toHaveLength(1);
  });

  /// The gap between the press and the first progress event is the whole signature check: with
  /// nothing said, the screen looked like the press had not registered.
  it("says something is underway before any progress arrives", async () => {
    answers({ update_install: () => new Promise(() => {}) });
    show(NEWER);
    await userEvent.click(screen.getByRole("button", { name: "Actualizar" }));

    expect(await screen.findByText("Preparando la actualización…")).toBeInTheDocument();
  });

  /// The Store installs without the window going anywhere, so the same button does the errand
  /// and the wording is the only thing that changes.
  it("installs a store copy from the store itself, without leaving the window", async () => {
    show(FROM_STORE);
    expect(screen.getByText(/La instala Microsoft Store/)).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Actualizar" }));
    expect(invoke).toHaveBeenCalledWith("update_install");
  });

  /// Every other route is taken down by its own installer, so only this one comes back to a
  /// window that is still standing — and what it comes back to has to be the truth.
  it("says the copy is current once the store is done with it", async () => {
    show(FROM_STORE);
    await userEvent.click(screen.getByRole("button", { name: "Actualizar" }));

    expect(await screen.findByText("Estás en la última versión")).toBeInTheDocument();
    expect(settled, "the progress has to be cleared or it hides the button").toHaveBeenCalled();
  });

  it("names the store while it installs, rather than talking about an installer", async () => {
    show(FROM_STORE, { stage: "installing", far: 100 });
    expect(screen.getByText(/Microsoft Store está instalando/)).toBeInTheDocument();
  });

  it("reports how far the download has got to a screen reader too", async () => {
    show(NEWER, { stage: "getting", far: 40 });
    expect(screen.getByText("Descargando… 40 %")).toBeInTheDocument();
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "40");
    expect(screen.queryByRole("button", { name: "Actualizar" })).toBeNull();
  });

  /// A cancelled store update stopped sending progress but never said so, and the button stayed
  /// hidden behind the stale progress for the rest of the session.
  it("clears the progress when an update fails, so it can be tried again", async () => {
    answers({ update_install: () => Promise.reject("updateStopped") });
    show(FROM_STORE);
    await userEvent.click(screen.getByRole("button", { name: "Actualizar" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("La actualización se detuvo");
    expect(settled).toHaveBeenCalled();
  });

  /// The refusals travel as catalogue keys, or a Spanish window shows English prose from Rust.
  it("says a refusal in the reader's language", async () => {
    answers({ update_install: () => Promise.reject("updateElsewhere") });
    show(NEWER);
    await userEvent.click(screen.getByRole("button", { name: "Actualizar" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "La dirección de descarga no es la de nuestras versiones",
    );
    expect(
      screen.getByRole("button", { name: "Actualizar" }),
      "a failed attempt has to be retryable",
    ).toBeEnabled();
  });

  /// A fault nobody wrote a sentence for is worth showing raw; swallowing it leaves a dead button.
  it("shows an unexpected fault as it came", async () => {
    answers({ update_install: () => Promise.reject("the disk is full") });
    show(NEWER);
    await userEvent.click(screen.getByRole("button", { name: "Actualizar" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("the disk is full");
  });

  it("clears a previous complaint when something new is tried", async () => {
    answers({ update_install: () => Promise.reject("updateBusy") });
    show(NEWER);
    await userEvent.click(screen.getByRole("button", { name: "Actualizar" }));
    expect(await screen.findByRole("alert")).toBeInTheDocument();

    answers();
    await userEvent.click(screen.getByRole("button", { name: "Actualizar" }));
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("offers the candidate track", async () => {
    show();
    const wants = await screen.findByRole("switch", { name: "Recibir versiones candidatas" });
    await userEvent.click(wants);

    expect(invoke).toHaveBeenCalledWith("update_candidates", { wants: true });
    expect(wants).toBeChecked();
  });

  /// The store carries no candidates, so offering the choice there would change nothing.
  it("keeps the candidate track from a copy the store keeps", async () => {
    answers({ about: () => Promise.resolve({ ...BUILD, candidatesApply: false }) });
    show();

    expect(await screen.findByText(/2\.0\.0/)).toBeInTheDocument();
    expect(screen.queryByRole("switch", { name: "Recibir versiones candidatas" })).toBeNull();
  });

  /// Changing track throws away what the old one found, so an offer left on screen would install
  /// nothing and complain about it.
  it("drops the standing offer when the track changes", async () => {
    show(NEWER);
    expect(screen.getByText("Versión 2.1.0 disponible")).toBeInTheDocument();

    await userEvent.click(
      await screen.findByRole("switch", { name: "Recibir versiones candidatas" }),
    );
    expect(await screen.findByText("Estás en la última versión")).toBeInTheDocument();
  });

  /// A switch left on after the backend refused would claim a track the copy is not on.
  it("puts the switch back when the choice is refused", async () => {
    answers({ update_candidates: () => Promise.reject("the file could not be written") });
    show();
    const wants = await screen.findByRole("switch", { name: "Recibir versiones candidatas" });
    await userEvent.click(wants);

    expect(await screen.findByRole("alert")).toHaveTextContent("could not be written");
    expect(wants).not.toBeChecked();
  });

  /// Two `<About>` in the DOM at once made an earlier version of this suite assert against the
  /// wrong tree, so the guard belongs here rather than in a comment.
  it("leaves nothing behind between renders", () => {
    show();
    cleanup();
    expect(screen.queryByText("Licencia GPL-3.0")).toBeNull();
  });
});
