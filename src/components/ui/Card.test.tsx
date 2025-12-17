import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "./Card";

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("Card", () => {
  it("renders children content", () => {
    render(<Card>Card content here</Card>);
    expect(screen.getByText("Card content here")).toBeDefined();
  });

  it("applies status border color when variant is status", () => {
    render(
      <Card variant="status" statusColor="#EF4444" data-testid="card">
        Status card
      </Card>
    );
    const card = screen.getByTestId("card");
    expect(card.style.borderLeftColor).toBe("rgb(239, 68, 68)");
  });

  it("composes with sub-components for structured layout", () => {
    render(
      <Card>
        <CardHeader>
          <CardTitle>Recording Settings</CardTitle>
          <CardDescription>Configure your microphone</CardDescription>
        </CardHeader>
        <CardContent>Settings form here</CardContent>
        <CardFooter>Save button here</CardFooter>
      </Card>
    );

    expect(screen.getByText("Recording Settings")).toBeDefined();
    expect(screen.getByText("Configure your microphone")).toBeDefined();
    expect(screen.getByText("Settings form here")).toBeDefined();
    expect(screen.getByText("Save button here")).toBeDefined();
  });
});
