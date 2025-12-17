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

describe("Card", () => {
  describe("variants", () => {
    it("renders standard variant with correct styles", () => {
      render(<Card data-testid="card">Content</Card>);
      const card = screen.getByTestId("card");
      expect(card.className).toContain("bg-white");
      expect(card.className).toContain("shadow-sm");
      expect(card.className).toContain("border");
      expect(card.className).toContain("hover:shadow-md");
    });

    it("renders interactive variant with cursor pointer", () => {
      render(
        <Card variant="interactive" data-testid="card">
          Content
        </Card>
      );
      const card = screen.getByTestId("card");
      expect(card.className).toContain("cursor-pointer");
      expect(card.className).toContain("hover:shadow-lg");
      expect(card.className).toContain("hover:border-heycat-orange");
    });

    it("renders status variant with left border", () => {
      render(
        <Card variant="status" data-testid="card">
          Content
        </Card>
      );
      const card = screen.getByTestId("card");
      expect(card.className).toContain("border-l-4");
    });

    it("applies status color to left border", () => {
      render(
        <Card variant="status" statusColor="#EF4444" data-testid="card">
          Content
        </Card>
      );
      const card = screen.getByTestId("card");
      expect(card.style.borderLeftColor).toBe("rgb(239, 68, 68)");
    });
  });

  describe("hover behavior", () => {
    it("has hover elevation classes for standard variant", () => {
      render(<Card data-testid="card">Content</Card>);
      const card = screen.getByTestId("card");
      expect(card.className).toContain("hover:shadow-md");
      expect(card.className).toContain("hover:border-neutral-300");
    });

    it("has enhanced hover for interactive variant", () => {
      render(
        <Card variant="interactive" data-testid="card">
          Content
        </Card>
      );
      const card = screen.getByTestId("card");
      expect(card.className).toContain("hover:shadow-lg");
    });
  });

  describe("composition", () => {
    it("renders full card composition", () => {
      render(
        <Card data-testid="card">
          <CardHeader data-testid="header">
            <CardTitle data-testid="title">Title</CardTitle>
            <CardDescription data-testid="description">Description</CardDescription>
          </CardHeader>
          <CardContent data-testid="content">Content</CardContent>
          <CardFooter data-testid="footer">Footer</CardFooter>
        </Card>
      );

      expect(screen.getByTestId("card")).toBeDefined();
      expect(screen.getByTestId("header")).toBeDefined();
      expect(screen.getByTestId("title").textContent).toBe("Title");
      expect(screen.getByTestId("description").textContent).toBe("Description");
      expect(screen.getByTestId("content").textContent).toBe("Content");
      expect(screen.getByTestId("footer").textContent).toBe("Footer");
    });
  });

  describe("CardHeader", () => {
    it("renders with margin bottom", () => {
      render(<CardHeader data-testid="header">Header</CardHeader>);
      expect(screen.getByTestId("header").className).toContain("mb-3");
    });
  });

  describe("CardTitle", () => {
    it("renders as h3 with correct typography", () => {
      render(<CardTitle data-testid="title">Title</CardTitle>);
      const title = screen.getByTestId("title");
      expect(title.tagName).toBe("H3");
      expect(title.className).toContain("text-lg");
      expect(title.className).toContain("font-semibold");
    });
  });

  describe("CardDescription", () => {
    it("renders with secondary text color", () => {
      render(<CardDescription data-testid="desc">Description</CardDescription>);
      const desc = screen.getByTestId("desc");
      expect(desc.className).toContain("text-sm");
      expect(desc.className).toContain("text-text-secondary");
    });
  });

  describe("CardFooter", () => {
    it("renders with flex layout", () => {
      render(<CardFooter data-testid="footer">Footer</CardFooter>);
      const footer = screen.getByTestId("footer");
      expect(footer.className).toContain("mt-4");
      expect(footer.className).toContain("flex");
      expect(footer.className).toContain("items-center");
    });
  });

  describe("custom className", () => {
    it("merges custom className with default styles", () => {
      render(
        <Card className="custom-class" data-testid="card">
          Content
        </Card>
      );
      const card = screen.getByTestId("card");
      expect(card.className).toContain("custom-class");
      expect(card.className).toContain("bg-white");
    });
  });
});
