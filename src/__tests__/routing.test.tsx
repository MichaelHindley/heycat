import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { RouterProvider, createMemoryRouter } from "react-router-dom";
import { router } from "../routes";
import { ToastProvider } from "../components/overlays";

// Mock Tauri APIs used by page components
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("@tauri-apps/plugin-store", () => ({
  load: vi.fn().mockResolvedValue({
    get: vi.fn().mockResolvedValue(null),
    set: vi.fn().mockResolvedValue(undefined),
    save: vi.fn().mockResolvedValue(undefined),
    onKeyChange: vi.fn().mockResolvedValue(() => {}),
  }),
}));

beforeEach(() => {
  vi.clearAllMocks();
});

/**
 * Helper to create a test router with a specific initial path.
 * Uses createMemoryRouter for testing to avoid hash router quirks.
 */
function createTestRouter(initialPath: string) {
  // Extract route definitions from the hash router
  const routes = router.routes;
  return createMemoryRouter(routes, {
    initialEntries: [initialPath],
  });
}

/**
 * Renders the router with required providers (ToastProvider).
 */
function renderWithProviders(testRouter: ReturnType<typeof createMemoryRouter>) {
  return render(
    <ToastProvider>
      <RouterProvider router={testRouter} />
    </ToastProvider>
  );
}

describe("Router Configuration", () => {
  it("/ renders Dashboard component", async () => {
    const testRouter = createTestRouter("/");
    renderWithProviders(testRouter);

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: /dashboard/i })).toBeInTheDocument();
    });
  });

  it("/commands renders Commands component", async () => {
    const testRouter = createTestRouter("/commands");
    renderWithProviders(testRouter);

    // The Commands page has heading "Voice Commands" - use text selector since we know it renders after loading
    await waitFor(() => {
      expect(screen.getByText(/voice commands/i, { selector: "h1" })).toBeInTheDocument();
    });
  });

  it("/recordings renders Recordings component", async () => {
    const testRouter = createTestRouter("/recordings");
    renderWithProviders(testRouter);

    // Wait for the page to load (indicated by the empty state or h1 appearing)
    await waitFor(() => {
      expect(screen.getByText(/recordings/i, { selector: "h1" })).toBeInTheDocument();
    });
  });

  it("/settings renders Settings component", async () => {
    const testRouter = createTestRouter("/settings");
    renderWithProviders(testRouter);

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: /settings/i })).toBeInTheDocument();
    });
  });

  it("unknown route redirects to / (Dashboard)", async () => {
    const testRouter = createTestRouter("/unknown-page");
    renderWithProviders(testRouter);

    // Should redirect to dashboard
    await waitFor(() => {
      expect(screen.getByRole("heading", { name: /dashboard/i })).toBeInTheDocument();
    });
  });
});
