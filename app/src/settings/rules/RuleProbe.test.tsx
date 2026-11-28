import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import RuleProbe from "./RuleProbe";
import type { Probe, RuleView } from "./types";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const RULE: RuleView = {
  id: "host:a.test",
  browser_id: "chrome",
  profile_id: null,
  kind: "host",
  covers: "a.test",
  browser: "Google Chrome",
  profile: null,
  icon: null,
  private: true,
  source_app: null,
  resolved: true,
  profile_gone: false,
};

const NEXT: RuleView = { ...RULE, id: "site:a.test", kind: "site", private: false };

function mount(over: Partial<Parameters<typeof RuleProbe>[0]> = {}) {
  const props = {
    rules: [RULE, NEXT],
    apps: [],
    onWinner: vi.fn(),
    onCreate: vi.fn(),
    onClose: vi.fn(),
    ...over,
  };
  render(<RuleProbe {...props} />);
  return props;
}

const typed = async (said: string) =>
  await userEvent.type(screen.getByRole("textbox", { name: "Enlace que quieres probar" }), said);

describe("rule probe", () => {
  beforeEach(() => invoke.mockReset());

  it("stops asking about an app once its last rule is gone", async () => {
    invoke.mockResolvedValue({ winner: null, why: null, also: [], site: "a.test" });
    const props = {
      rules: [RULE],
      apps: ["slack"],
      onWinner: vi.fn(),
      onCreate: vi.fn(),
      onClose: vi.fn(),
    };
    const { rerender } = render(<RuleProbe {...props} />);
    await userEvent.selectOptions(screen.getByRole("combobox"), "slack");
    await typed("a.test");
    await vi.waitFor(() =>
      expect(invoke).toHaveBeenLastCalledWith("rules_probe", { url: "a.test", fromApp: "slack" }),
    );
    rerender(<RuleProbe {...props} rules={[NEXT]} apps={[]} />);
    await vi.waitFor(() =>
      expect(invoke).toHaveBeenLastCalledWith("rules_probe", { url: "a.test", fromApp: null }),
    );
  });

  it("puts the cursor in the box and asks nothing until something is typed", () => {
    mount();
    expect(screen.getByRole("textbox", { name: "Enlace que quieres probar" })).toHaveFocus();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("tells the winner to the list and says a private window will be used", async () => {
    const probe: Probe = {
      winner: RULE,
      why: "narrower",
      also: [NEXT],
      site: "a.test",
      file_type: null,
    };
    invoke.mockResolvedValue(probe);
    const { onWinner } = mount();
    await typed("a.test");
    expect(await screen.findByRole("status")).toHaveTextContent("privada");
    expect(onWinner).toHaveBeenLastCalledWith("host:a.test");
  });

  it("asks again when the rules change under it", async () => {
    invoke.mockResolvedValue({ winner: RULE, why: "alone", also: [], site: "a.test" });
    const props = {
      rules: [RULE],
      apps: [],
      onWinner: vi.fn(),
      onCreate: vi.fn(),
      onClose: vi.fn(),
    };
    const { rerender } = render(<RuleProbe {...props} />);
    await typed("a.test");
    await screen.findByRole("status");
    const before = invoke.mock.calls.length;
    rerender(<RuleProbe {...props} rules={[RULE, NEXT]} />);
    await vi.waitFor(() => expect(invoke.mock.calls).toHaveLength(before + 1));
  });

  it("drops an answer that arrives after the box moved on", async () => {
    let late: (p: Probe) => void = () => {};
    invoke
      .mockImplementationOnce(() => new Promise<Probe>((ok) => (late = ok)))
      .mockResolvedValueOnce({
        winner: null,
        why: null,
        also: [],
        site: "b.test",
        file_type: null,
      });
    const { onWinner } = mount();
    await typed("a");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledTimes(1));
    await typed("b");
    expect(await screen.findByText("Ninguna regla coincide")).toBeInTheDocument();
    late({ winner: RULE, why: "alone", also: [], site: "a.test", file_type: null });
    await Promise.resolve();
    expect(screen.queryByRole("status")).toHaveTextContent("Ninguna regla coincide");
    expect(onWinner).not.toHaveBeenCalledWith("host:a.test");
  });

  it("offers a rule for the site of a link, and for the type of a document", async () => {
    invoke.mockResolvedValueOnce({
      winner: null,
      why: null,
      also: [],
      site: "a.test",
      file_type: null,
    });
    const { onCreate } = mount();
    await typed("a.test");
    await userEvent.click(
      await screen.findByRole("button", { name: "Crear una regla para a.test" }),
    );
    expect(onCreate).toHaveBeenLastCalledWith({ kind: "site", value: "a.test" });

    invoke.mockResolvedValue({ winner: null, why: null, also: [], site: "", file_type: "pdf" });
    await typed("/x.pdf");
    await userEvent.click(await screen.findByRole("button", { name: "Crear una regla para .pdf" }));
    expect(onCreate).toHaveBeenLastCalledWith({ kind: "file_type", value: "pdf" });
  });

  it("closes from its own button and from Escape anywhere inside it", async () => {
    const { onClose } = mount();
    await userEvent.click(screen.getByRole("button", { name: "Cerrar la prueba" }));
    expect(onClose).toHaveBeenCalledTimes(1);
    await userEvent.type(
      screen.getByRole("textbox", { name: "Enlace que quieres probar" }),
      "{Escape}",
    );
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it("offers the applications the rules name, and sends none by default", async () => {
    invoke.mockResolvedValue({ winner: RULE, why: "alone", also: [], site: "a.test" });
    mount({ apps: ["slack", "teams"] });
    expect(screen.getByRole("option", { name: "cualquier aplicación" })).toBeInTheDocument();
    await typed("a.test");
    await vi.waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("rules_probe", { url: "a.test", fromApp: null }),
    );
  });
});
