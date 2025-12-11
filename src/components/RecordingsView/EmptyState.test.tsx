import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { EmptyState } from "./EmptyState";

describe("EmptyState", () => {
  describe("when no recordings exist (hasFiltersActive=false)", () => {
    it('displays "No recordings yet" message', () => {
      render(<EmptyState hasFiltersActive={false} />);

      expect(screen.getByText("No recordings yet")).toBeDefined();
    });

    it("displays helpful description to make first recording", () => {
      render(<EmptyState hasFiltersActive={false} />);

      expect(
        screen.getByText("Make your first recording to see it here")
      ).toBeDefined();
    });

    it("shows microphone icon", () => {
      render(<EmptyState hasFiltersActive={false} />);

      const icon = document.querySelector(".empty-state__icon");
      expect(icon?.textContent).toContain("🎙️");
    });

    it("does not show clear filters button", () => {
      render(<EmptyState hasFiltersActive={false} />);

      expect(screen.queryByRole("button")).toBeNull();
    });
  });

  describe("when filters match no recordings (hasFiltersActive=true)", () => {
    it('displays "No recordings match your filters" message', () => {
      render(<EmptyState hasFiltersActive={true} />);

      expect(screen.getByText("No recordings match your filters")).toBeDefined();
    });

    it("displays helpful description about adjusting filters", () => {
      render(<EmptyState hasFiltersActive={true} />);

      expect(
        screen.getByText("Try adjusting your filters to see more results")
      ).toBeDefined();
    });

    it("shows search icon", () => {
      render(<EmptyState hasFiltersActive={true} />);

      const icon = document.querySelector(".empty-state__icon");
      expect(icon?.textContent).toContain("🔍");
    });

    it("shows clear filters button when onClearFilters is provided", () => {
      const onClearFilters = vi.fn();
      render(
        <EmptyState hasFiltersActive={true} onClearFilters={onClearFilters} />
      );

      expect(screen.getByRole("button", { name: "Clear filters" })).toBeDefined();
    });

    it("calls onClearFilters when clear button is clicked", () => {
      const onClearFilters = vi.fn();
      render(
        <EmptyState hasFiltersActive={true} onClearFilters={onClearFilters} />
      );

      fireEvent.click(screen.getByRole("button", { name: "Clear filters" }));

      expect(onClearFilters).toHaveBeenCalledTimes(1);
    });

    it("does not show clear button when onClearFilters is not provided", () => {
      render(<EmptyState hasFiltersActive={true} />);

      expect(screen.queryByRole("button")).toBeNull();
    });
  });

  describe("after clearing filters", () => {
    it("shows correct state after clearing filters (no recordings)", () => {
      const { rerender } = render(<EmptyState hasFiltersActive={true} />);

      expect(screen.getByText("No recordings match your filters")).toBeDefined();

      rerender(<EmptyState hasFiltersActive={false} />);

      expect(screen.getByText("No recordings yet")).toBeDefined();
    });
  });

  describe("accessibility", () => {
    it("has ARIA live region for status updates", () => {
      render(<EmptyState hasFiltersActive={false} />);

      const status = screen.getByRole("status");
      expect(status.getAttribute("aria-live")).toBe("polite");
    });

    it("hides icon from screen readers", () => {
      render(<EmptyState hasFiltersActive={false} />);

      const icon = document.querySelector(".empty-state__icon");
      expect(icon?.getAttribute("aria-hidden")).toBe("true");
    });

    it("applies custom className", () => {
      render(<EmptyState hasFiltersActive={false} className="custom-class" />);

      const status = screen.getByRole("status");
      expect(status.className).toContain("custom-class");
    });
  });

  describe("rendering", () => {
    it("renders without errors when hasFiltersActive is false", () => {
      expect(() => render(<EmptyState hasFiltersActive={false} />)).not.toThrow();
    });

    it("renders without errors when hasFiltersActive is true", () => {
      expect(() => render(<EmptyState hasFiltersActive={true} />)).not.toThrow();
    });

    it("renders without errors with all props", () => {
      expect(() =>
        render(
          <EmptyState
            hasFiltersActive={true}
            onClearFilters={() => {}}
            className="test"
          />
        )
      ).not.toThrow();
    });
  });
});
