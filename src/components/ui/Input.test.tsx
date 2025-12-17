import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Input, Textarea, Label, FormField } from "./Input";

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("Input", () => {
  it("accepts user text input", async () => {
    const user = userEvent.setup();
    const handleChange = vi.fn();
    render(<Input onChange={handleChange} placeholder="Enter name" />);

    await user.type(screen.getByPlaceholderText("Enter name"), "Hello");
    expect(handleChange).toHaveBeenCalled();
  });

  it("cannot be edited when disabled", async () => {
    const user = userEvent.setup();
    const handleChange = vi.fn();
    render(<Input disabled onChange={handleChange} placeholder="Enter name" />);

    await user.type(screen.getByPlaceholderText("Enter name"), "Hello");
    expect(handleChange).not.toHaveBeenCalled();
  });
});

describe("FormField", () => {
  it("displays error message when validation fails", () => {
    render(
      <FormField error="This field is required">
        <Input placeholder="Email" />
      </FormField>
    );

    const errorMessage = screen.getByRole("alert");
    expect(errorMessage.textContent).toBe("This field is required");
  });
});

describe("Label", () => {
  it("shows required indicator when field is required", () => {
    render(<Label required>Email address</Label>);
    expect(screen.getByText("*")).toBeDefined();
  });
});
