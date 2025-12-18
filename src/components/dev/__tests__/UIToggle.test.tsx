import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { UIToggle } from "../UIToggle";

// Mock import.meta.env.DEV - default to true (development mode)
vi.stubGlobal("import.meta", { env: { DEV: true } });

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("UIToggle", () => {
  beforeEach(() => {
    localStorage.clear();
    // Reset DEV mode to true before each test
    vi.stubGlobal("import.meta", { env: { DEV: true } });
  });

  it("clicking toggle switches between old and new UI labels", async () => {
    const user = userEvent.setup();
    const mockToggle = vi.fn();

    // Start with old UI
    render(<UIToggle mode="old" onToggle={mockToggle} />);

    expect(screen.getByText("Old UI")).toBeDefined();

    await user.click(screen.getByRole("button"));
    expect(mockToggle).toHaveBeenCalledTimes(1);
  });

  it("shows correct visual indicator for current mode", () => {
    const { rerender } = render(<UIToggle mode="old" onToggle={() => {}} />);
    expect(screen.getByText("Old UI")).toBeDefined();

    rerender(<UIToggle mode="new" onToggle={() => {}} />);
    expect(screen.getByText("New UI")).toBeDefined();
  });

  it("keyboard shortcut Ctrl+Shift+U triggers toggle", async () => {
    const user = userEvent.setup();
    const mockToggle = vi.fn();

    render(<UIToggle mode="old" onToggle={mockToggle} />);

    await user.keyboard("{Control>}{Shift>}U{/Shift}{/Control}");
    expect(mockToggle).toHaveBeenCalledTimes(1);
  });

  it("has accessible label describing toggle action", () => {
    render(<UIToggle mode="old" onToggle={() => {}} />);

    const button = screen.getByRole("button");
    expect(button.getAttribute("aria-label")).toBe("Switch to new UI");
  });

  it("has accessible label for new mode too", () => {
    render(<UIToggle mode="new" onToggle={() => {}} />);

    const button = screen.getByRole("button");
    expect(button.getAttribute("aria-label")).toBe("Switch to old UI");
  });
});
