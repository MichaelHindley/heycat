import { renderHook, act } from "@testing-library/react";
import { describe, expect, it, beforeEach } from "vitest";
import { useUIMode } from "./useUIMode";

// Tests focus on user-visible behavior per TESTING.md guidelines

describe("useUIMode", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("defaults to old UI when no preference is stored", () => {
    const { result } = renderHook(() => useUIMode());
    expect(result.current.mode).toBe("old");
  });

  it("persists mode preference to localStorage and retrieves it", () => {
    // First render - set mode to new
    const { result, unmount } = renderHook(() => useUIMode());

    act(() => {
      result.current.setMode("new");
    });

    expect(result.current.mode).toBe("new");
    expect(localStorage.getItem("heycat-ui-mode")).toBe("new");

    // Unmount and re-render to simulate page reload
    unmount();

    const { result: result2 } = renderHook(() => useUIMode());
    expect(result2.current.mode).toBe("new");
  });

  it("toggle() switches between old and new modes", () => {
    const { result } = renderHook(() => useUIMode());

    expect(result.current.mode).toBe("old");

    act(() => {
      result.current.toggle();
    });
    expect(result.current.mode).toBe("new");

    act(() => {
      result.current.toggle();
    });
    expect(result.current.mode).toBe("old");
  });
});
