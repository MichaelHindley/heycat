import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Select, SelectItem } from "./Select";

// Note: Radix UI Select uses pointer capture APIs that jsdom doesn't fully support.
// Tests focus on initial render and accessibility. Interactive behavior verified via E2E.

describe("Select", () => {
  it("renders with accessible combobox role", () => {
    render(
      <Select placeholder="Select an option">
        <SelectItem value="option1">Option 1</SelectItem>
      </Select>
    );
    expect(screen.getByRole("combobox")).toBeDefined();
  });

  it("displays placeholder when no value selected", () => {
    render(
      <Select placeholder="Choose one">
        <SelectItem value="a">A</SelectItem>
      </Select>
    );
    expect(screen.getByText("Choose one")).toBeDefined();
  });

  it("displays selected value in trigger", () => {
    render(
      <Select defaultValue="option1">
        <SelectItem value="option1">First Option</SelectItem>
        <SelectItem value="option2">Second Option</SelectItem>
      </Select>
    );
    expect(screen.getByRole("combobox").textContent).toContain("First Option");
  });

  it("is disabled when disabled prop is true", () => {
    render(
      <Select placeholder="Select..." disabled>
        <SelectItem value="a">A</SelectItem>
      </Select>
    );
    expect(screen.getByRole("combobox").getAttribute("data-disabled")).toBe("");
  });
});
