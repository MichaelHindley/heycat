import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Button } from "./Button";

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("Button", () => {
  it("triggers onClick handler when clicked", async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>Submit</Button>);

    await user.click(screen.getByRole("button", { name: "Submit" }));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it("cannot be clicked when disabled", async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();
    render(
      <Button onClick={handleClick} disabled>
        Submit
      </Button>
    );

    await user.click(screen.getByRole("button", { name: "Submit" }));
    expect(handleClick).not.toHaveBeenCalled();
    expect(screen.getByRole("button").hasAttribute("disabled")).toBe(true);
  });

  it("shows loading spinner and prevents interaction during loading state", async () => {
    const user = userEvent.setup();
    const handleClick = vi.fn();
    render(
      <Button onClick={handleClick} loading>
        Submit
      </Button>
    );

    const button = screen.getByRole("button");
    expect(button.hasAttribute("disabled")).toBe(true);
    expect(screen.getByTestId("button-spinner")).toBeDefined();

    await user.click(button);
    expect(handleClick).not.toHaveBeenCalled();
  });

  it("renders as anchor when using asChild pattern", () => {
    render(
      <Button asChild>
        <a href="/settings">Settings</a>
      </Button>
    );

    const link = screen.getByRole("link", { name: "Settings" });
    expect(link.getAttribute("href")).toBe("/settings");
  });
});
