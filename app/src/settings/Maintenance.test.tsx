import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Maintenance from "./Maintenance";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

describe("maintenance", () => {
  beforeEach(() => {
    invoke.mockReset();
    invoke.mockResolvedValue(null);
  });

  /// Everything on this screen destroys something, so no button may act on the
  /// first click.
  it("asks before wiping the configuration, and says what is lost", async () => {
    render(<Maintenance />);
    await userEvent.click(screen.getByRole("button", { name: "Restablecer" }));
    expect(invoke).not.toHaveBeenCalled();
    expect(await screen.findByText(/No se puede deshacer/)).toBeInTheDocument();
  });

  it("does nothing when the question is dismissed", async () => {
    render(<Maintenance />);
    await userEvent.click(screen.getByRole("button", { name: "Restablecer" }));
    await userEvent.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("wipes the configuration once confirmed", async () => {
    render(<Maintenance />);
    await userEvent.click(screen.getByRole("button", { name: "Restablecer" }));
    const ask = await screen.findByRole("alertdialog", { name: "Restablecer la configuración" });
    await userEvent.click(within(ask).getByRole("button", { name: "Restablecer" }));
    expect(invoke).toHaveBeenCalledWith("maintenance_reset", {});
  });

  it("explains that a rescan keeps what the user added by hand", async () => {
    render(<Maintenance />);
    await userEvent.click(screen.getByRole("button", { name: "Buscar" }));
    expect(await screen.findByText(/añadiste a mano se conservan/)).toBeInTheDocument();
  });

  it("unregisters through the same command the links screen uses", async () => {
    render(<Maintenance />);
    await userEvent.click(screen.getByRole("button", { name: "Quitar" }));
    const ask = await screen.findByRole("alertdialog");
    await userEvent.click(within(ask).getByRole("button", { name: "Quitar" }));
    expect(invoke).toHaveBeenCalledWith("system_set_registered", { enabled: false });
  });

  /// The report is meant to be pasted into a public issue, so the screen has to
  /// promise what the core actually redacts.
  it("saves a report and says where it landed", async () => {
    invoke.mockResolvedValue("C:/Users/x/Desktop/linkunbound-diagnostico.md");
    render(<Maintenance />);
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(invoke).toHaveBeenCalledWith("maintenance_report");
    expect(await screen.findByText(/linkunbound-diagnostico\.md/)).toBeInTheDocument();
  });

  it("saves the report without asking, since it destroys nothing", async () => {
    render(<Maintenance />);
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(screen.queryByRole("alertdialog")).toBeNull();
  });

  it("surfaces a refusal instead of claiming it worked", async () => {
    invoke.mockRejectedValue("the registry refused the write");
    render(<Maintenance />);
    await userEvent.click(screen.getByRole("button", { name: "Restablecer" }));
    const ask = await screen.findByRole("alertdialog");
    await userEvent.click(within(ask).getByRole("button", { name: "Restablecer" }));
    expect(await screen.findByText(/registry refused/)).toBeInTheDocument();
  });
});
