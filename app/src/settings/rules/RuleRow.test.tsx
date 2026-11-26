import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import RuleRow from "./RuleRow";
import type { Destination, RuleView } from "./types";

const RULE: RuleView = {
  id: "site:github.com",
  browser_id: "chrome",
  profile_id: "Default",
  kind: "site",
  covers: "github.com",
  browser: "Google Chrome",
  profile: "Casa",
  icon: "data:image/png;base64,AA==",
  private: false,
  source_app: null,
  resolved: true,
  profile_gone: false,
};

const CHOICES: Destination[] = [
  {
    browserId: "chrome",
    profileId: "Default",
    label: "Chrome · Casa",
    icon: null,
    opensPrivately: true,
  },
  { browserId: "firefox", profileId: null, label: "Firefox", icon: null, opensPrivately: true },
];

const inList = (ui: React.ReactElement) => render(<ul>{ui}</ul>);

describe("rule row", () => {
  it("shows the kind, the full value in its title, and where it opens", () => {
    const long = `https://example.org/${"a".repeat(120)}`;
    inList(<RuleRow rule={{ ...RULE, kind: "url", covers: long }} />);
    const row = screen.getByRole("listitem");
    expect(within(row).getByText("Esta URL")).toBeInTheDocument();
    expect(within(row).getByTitle(long)).toHaveClass("truncate");
    expect(within(row).getByText("Google Chrome")).toBeInTheDocument();
    expect(within(row).getByText("Casa")).toBeInTheDocument();
  });

  it("marks every place the search found, whatever the case, and nothing else", () => {
    const { container } = inList(
      <RuleRow rule={{ ...RULE, covers: "git.github.com/Git" }} query=" GIT " />,
    );
    const marks = [...container.querySelectorAll("mark")].map((m) => m.textContent);
    expect(marks).toEqual(["git", "git", "Git"]);
    expect(screen.getByTitle("git.github.com/Git")).toHaveTextContent("git.github.com/Git");
  });

  it("names the app of a rule the picker bound to a site and an app at once", () => {
    inList(<RuleRow rule={{ ...RULE, source_app: "slack" }} query="sla" />);
    const value = screen.getByTitle("github.com · slack");
    expect(value).toHaveTextContent("github.com · desde slack");
    expect(value.querySelector("mark")).toHaveTextContent("sla");
  });

  it("marks nothing when the search is blank", () => {
    const { container } = inList(<RuleRow rule={RULE} query="   " />);
    expect(container.querySelector("mark")).toBeNull();
  });

  it("tags the rule that wins and the ones that also match", () => {
    inList(<RuleRow rule={RULE} verdict="wins" />);
    expect(screen.getByText("Gana")).toBeInTheDocument();
    inList(<RuleRow rule={{ ...RULE, id: "x" }} verdict="also" />);
    expect(screen.getByText("También coincide")).toBeInTheDocument();
  });

  it("is read-only without the handlers of the list: no buttons, no picker", () => {
    inList(<RuleRow rule={{ ...RULE, resolved: false }} />);
    expect(screen.queryByRole("button")).toBeNull();
    expect(screen.queryByText(/ya no está instalado/)).toBeNull();
  });

  it("offers edit and delete, each named after what the rule covers", async () => {
    const onEdit = vi.fn();
    const onRemove = vi.fn();
    inList(
      <RuleRow
        rule={RULE}
        choices={CHOICES}
        onRetarget={vi.fn()}
        onEdit={onEdit}
        onRemove={onRemove}
      />,
    );
    await userEvent.click(screen.getByRole("button", { name: "Editar la regla de github.com" }));
    await userEvent.click(screen.getByRole("button", { name: "Eliminar la regla de github.com" }));
    expect(onEdit).toHaveBeenCalledOnce();
    expect(onRemove).toHaveBeenCalledOnce();
  });

  it("names a rule for an app by the app", () => {
    inList(
      <RuleRow
        rule={{ ...RULE, kind: "any", covers: "", source_app: "teams" }}
        choices={CHOICES}
        onRetarget={vi.fn()}
        onEdit={vi.fn()}
        onRemove={vi.fn()}
      />,
    );
    expect(screen.getByTitle("teams")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Editar la regla de teams" })).toBeInTheDocument();
  });

  it("marks a private rule", () => {
    inList(<RuleRow rule={{ ...RULE, private: true }} />);
    expect(screen.getByText("privada")).toBeInTheDocument();
  });

  it("explains a rule whose browser is gone and offers to choose another", async () => {
    const onPicking = vi.fn();
    inList(
      <RuleRow
        rule={{ ...RULE, resolved: false, browser: "brave" }}
        choices={CHOICES}
        onPicking={onPicking}
        onRetarget={vi.fn()}
        onEdit={vi.fn()}
        onRemove={vi.fn()}
      />,
    );
    expect(screen.getByText(/brave ya no está instalado/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Elegir otro navegador" }));
    expect(onPicking).toHaveBeenCalledWith(true);
  });

  it("gives the broken rule's pick button a hover and a focus ring", () => {
    inList(
      <RuleRow
        rule={{ ...RULE, resolved: false, browser: "brave" }}
        choices={CHOICES}
        onPicking={vi.fn()}
        onRetarget={vi.fn()}
        onEdit={vi.fn()}
        onRemove={vi.fn()}
      />,
    );
    const pick = screen.getByRole("button", { name: "Elegir otro navegador" });
    expect(pick.className).toContain("hover:");
    expect(pick.className).toContain("focus-visible:outline");
  });

  it("paints a broken rule with caution, never danger", () => {
    inList(
      <RuleRow
        rule={{ ...RULE, resolved: false, browser: "brave" }}
        choices={CHOICES}
        onPicking={vi.fn()}
        onEdit={vi.fn()}
        onRemove={vi.fn()}
      />,
    );
    expect(screen.getByRole("listitem")).toHaveClass("bg-caution/[0.08]");
    expect(screen.getByText(/brave ya no está instalado/).parentElement).toHaveClass(
      "text-caution",
    );
    const pick = screen.getByRole("button", { name: "Elegir otro navegador" });
    expect(pick).toHaveClass("bg-caution", "text-on-accent");
    expect(pick.className).not.toContain("danger");
  });

  it("colours the app chip with caution and the site chip grey with ink-2", () => {
    const { rerender } = inList(
      <RuleRow rule={{ ...RULE, kind: "any", covers: "", source_app: "teams" }} />,
    );
    expect(screen.getByText("Desde")).toHaveClass("text-caution");
    rerender(
      <ul>
        <RuleRow rule={RULE} />
      </ul>,
    );
    expect(screen.getByText("Sitio")).toHaveClass("text-ink-2");
  });

  it("tags the winner on accent text and the others in ink-2", () => {
    inList(<RuleRow rule={RULE} verdict="wins" />);
    expect(screen.getByText("Gana")).toHaveClass("bg-accent", "text-on-accent");
    inList(<RuleRow rule={{ ...RULE, id: "x" }} verdict="also" />);
    expect(screen.getByText("También coincide")).toHaveClass("text-ink-2");
  });

  it("blames the profile, not the browser, when only the profile is gone", () => {
    inList(
      <RuleRow
        rule={{ ...RULE, resolved: false, profile_gone: true, profile: null }}
        choices={CHOICES}
        onPicking={vi.fn()}
        onRetarget={vi.fn()}
        onEdit={vi.fn()}
        onRemove={vi.fn()}
      />,
    );
    expect(
      screen.getByText(/El perfil de Google Chrome que usaba esta regla ya no está/),
    ).toBeInTheDocument();
    expect(screen.queryByText(/ya no está instalado/)).toBeNull();
  });

  it("offers no way to choose another browser when there is none to choose", () => {
    inList(
      <RuleRow
        rule={{ ...RULE, resolved: false, browser: "brave" }}
        choices={[]}
        onPicking={vi.fn()}
        onRetarget={vi.fn()}
        onEdit={vi.fn()}
        onRemove={vi.fn()}
      />,
    );
    expect(screen.getByText(/brave ya no está instalado/)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Elegir otro navegador" })).toBeNull();
  });

  it("opens its picker when the list says so", () => {
    inList(
      <RuleRow rule={RULE} choices={CHOICES} picking onPicking={vi.fn()} onRetarget={vi.fn()} />,
    );
    expect(screen.getByRole("listbox")).toBeInTheDocument();
  });
});
