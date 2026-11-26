import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import DestinationPicker from "./DestinationPicker";
import { type Destination, named } from "./types";

const CHOICES: Destination[] = [
  {
    browserId: "chrome",
    profileId: "Default",
    label: "Chrome · Casa",
    icon: null,
    opensPrivately: true,
  },
  {
    browserId: "plain",
    profileId: null,
    label: "Plain",
    icon: "data:image/png;base64,AA==",
    opensPrivately: false,
  },
  { browserId: "firefox", profileId: null, label: "Firefox", icon: null, opensPrivately: true },
];

function Picker({
  onChange = () => {},
  needsPrivate = false,
  value = named("chrome", "Default"),
}: {
  onChange?: (key: string) => void;
  needsPrivate?: boolean;
  value?: string;
}) {
  return (
    <>
      <DestinationPicker
        label="Abrir en"
        choices={CHOICES}
        value={value}
        face={<span>cara</span>}
        needsPrivate={needsPrivate}
        onChange={onChange}
      />
      <button type="button">fuera</button>
    </>
  );
}

function Held() {
  const [open, setOpen] = useState(true);
  return (
    <DestinationPicker
      label="Abrir en"
      choices={CHOICES}
      value={named("chrome", "Default")}
      face="cara"
      open={open}
      onOpenChange={setOpen}
      onChange={() => {}}
    />
  );
}

const opener = () => screen.getByRole("button", { name: "Abrir en" });

describe("destination picker", () => {
  /// A row's picker sits at the right edge of the window; opened to the right it was cut off.
  it("opens a row's list towards the left and a form's towards the right", () => {
    const { unmount } = render(
      <DestinationPicker
        inline
        open
        label="Abrir en"
        choices={CHOICES}
        value=""
        face="cara"
        onChange={() => {}}
      />,
    );
    expect(screen.getByRole("listbox")).toHaveClass("right-0");
    unmount();
    render(
      <DestinationPicker
        open
        label="Abrir en"
        choices={CHOICES}
        value=""
        face="cara"
        onChange={() => {}}
      />,
    );
    expect(screen.getByRole("listbox")).toHaveClass("left-0");
  });

  it("is a button that announces a listbox, closed until asked", () => {
    render(<Picker />);
    expect(opener()).toHaveAttribute("aria-haspopup", "listbox");
    expect(opener()).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  it("opens on a click, marks the chosen option and focuses the list", async () => {
    render(<Picker />);
    await userEvent.click(opener());
    expect(opener()).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByRole("listbox", { name: "Abrir en" })).toHaveFocus();
    expect(screen.getByRole("option", { name: /Chrome · Casa/ })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByRole("option", { name: /Firefox/ })).toHaveAttribute(
      "aria-selected",
      "false",
    );
  });

  it("says which browsers have no private window", async () => {
    render(<Picker />);
    await userEvent.click(opener());
    expect(screen.getByRole("option", { name: /Plain/ })).toHaveTextContent("sin ventana privada");
    expect(screen.getByRole("option", { name: /Firefox/ })).not.toHaveTextContent("sin ventana");
  });

  it("chooses with the mouse and gives the focus back to the button", async () => {
    const onChange = vi.fn();
    render(<Picker onChange={onChange} />);
    await userEvent.click(opener());
    await userEvent.click(screen.getByRole("option", { name: /Firefox/ }));
    expect(onChange).toHaveBeenCalledWith(named("firefox", null));
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(opener()).toHaveFocus();
  });

  it("moves with the arrows and chooses with Enter", async () => {
    const onChange = vi.fn();
    render(<Picker onChange={onChange} />);
    await userEvent.click(opener());
    await userEvent.keyboard("{ArrowDown}{ArrowDown}{Enter}");
    expect(onChange).toHaveBeenCalledWith(named("firefox", null));
  });

  it("stops at either end instead of wrapping", async () => {
    const onChange = vi.fn();
    render(<Picker onChange={onChange} />);
    await userEvent.click(opener());
    await userEvent.keyboard("{ArrowUp}{ArrowUp}{Enter}");
    expect(onChange).toHaveBeenCalledWith(named("chrome", "Default"));
  });

  it("jumps to the first and the last with Home and End, and chooses with Space", async () => {
    const onChange = vi.fn();
    render(<Picker onChange={onChange} value={named("plain", null)} />);
    await userEvent.click(opener());
    await userEvent.keyboard("{End}");
    expect(screen.getByRole("listbox")).toHaveAttribute(
      "aria-activedescendant",
      screen.getByRole("option", { name: /Firefox/ }).id,
    );
    await userEvent.keyboard("{Home} ");
    expect(onChange).toHaveBeenCalledWith(named("chrome", "Default"));
  });

  it("closes on Escape without choosing", async () => {
    const onChange = vi.fn();
    render(<Picker onChange={onChange} />);
    await userEvent.click(opener());
    await userEvent.keyboard("{ArrowDown}{Escape}");
    expect(onChange).not.toHaveBeenCalled();
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(opener()).toHaveFocus();
  });

  it("closes when the focus leaves with Tab or the mouse goes elsewhere", async () => {
    render(<Picker />);
    await userEvent.click(opener());
    await userEvent.keyboard("{Tab}");
    expect(screen.queryByRole("listbox")).toBeNull();

    await userEvent.click(opener());
    await userEvent.click(screen.getByRole("button", { name: "fuera" }));
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  it("closes when the button is pressed again", async () => {
    render(<Picker />);
    await userEvent.click(opener());
    await userEvent.click(opener());
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  describe("for a rule that opens privately", () => {
    it("greys a browser without a private window and cannot choose it", async () => {
      const onChange = vi.fn();
      render(<Picker needsPrivate onChange={onChange} />);
      await userEvent.click(opener());
      const plain = screen.getByRole("option", { name: /Plain/ });
      expect(plain).toHaveAttribute("aria-disabled", "true");
      await userEvent.click(plain);
      expect(onChange).not.toHaveBeenCalled();
      expect(screen.getByRole("listbox")).toBeInTheDocument();
    });

    it("skips what is greyed when moving with the keys", async () => {
      const onChange = vi.fn();
      render(<Picker needsPrivate onChange={onChange} />);
      await userEvent.click(opener());
      await userEvent.keyboard("{ArrowDown}{Enter}");
      expect(onChange).toHaveBeenCalledWith(named("firefox", null));
    });

    it("starts on the first usable one when the rule points at none of them", async () => {
      const onChange = vi.fn();
      render(<Picker needsPrivate onChange={onChange} value={named("gone", null)} />);
      await userEvent.click(opener());
      await userEvent.keyboard("{Enter}");
      expect(onChange).toHaveBeenCalledWith(named("chrome", "Default"));
    });
  });

  it("can be opened and closed by whoever holds it", async () => {
    render(<Held />);
    expect(screen.getByRole("listbox")).toBeInTheDocument();
    await userEvent.keyboard("{Escape}");
    expect(screen.queryByRole("listbox")).toBeNull();
  });
});
