import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TranscriptionSettings } from "./TranscriptionSettings";

// Mock invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock listen
const mockListeners: Map<string, ((event: { payload: unknown }) => void)[]> = new Map();
const mockListen = vi.fn().mockImplementation((eventName: string, callback: (event: { payload: unknown }) => void) => {
  const listeners = mockListeners.get(eventName) || [];
  listeners.push(callback);
  mockListeners.set(eventName, listeners);
  return Promise.resolve(() => {
    const currentListeners = mockListeners.get(eventName) || [];
    const index = currentListeners.indexOf(callback);
    if (index > -1) {
      currentListeners.splice(index, 1);
    }
  });
});

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

function emitEvent(eventName: string, payload: unknown) {
  const listeners = mockListeners.get(eventName) || [];
  listeners.forEach((cb) => cb({ payload }));
}

describe("TranscriptionSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListeners.clear();

    // Default mocks
    mockInvoke.mockImplementation((cmd: string, args?: unknown) => {
      if (cmd === "check_parakeet_model_status") {
        return Promise.resolve(false);
      }
      if (cmd === "get_transcription_mode") {
        return Promise.resolve("batch");
      }
      return Promise.resolve();
    });
  });

  describe("Component rendering", () => {
    it("renders with TDT model section visible", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Transcription")).toBeDefined();
      });

      expect(screen.getByText("Batch (TDT)")).toBeDefined();
    });

    it("displays model description", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("High-accuracy transcription after recording completes")).toBeDefined();
      });
    });

    it("applies custom className", async () => {
      const { container } = render(<TranscriptionSettings className="custom-class" />);

      await waitFor(() => {
        expect(container.querySelector(".transcription-settings.custom-class")).toBeDefined();
      });
    });
  });

  describe("Model download functionality", () => {
    it("TDT download button triggers download_model with model_type='tdt'", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      const tdtCard = screen.getByText("Batch (TDT)").closest(".transcription-settings__model-card");
      const downloadButton = tdtCard?.querySelector("button");

      await userEvent.click(downloadButton!);

      expect(mockInvoke).toHaveBeenCalledWith("download_model", { modelType: "tdt" });
    });

    it("progress bar updates when model_file_download_progress event is received", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      // Start download
      const tdtCard = screen.getByText("Batch (TDT)").closest(".transcription-settings__model-card");
      const downloadButton = tdtCard?.querySelector("button");
      await userEvent.click(downloadButton!);

      // Emit progress event
      emitEvent("model_file_download_progress", {
        modelType: "tdt",
        fileName: "model.bin",
        percent: 50,
        bytesDownloaded: 500,
        totalBytes: 1000,
      });

      await waitFor(() => {
        const progressBar = screen.getByRole("progressbar", { name: "Batch (TDT) download progress" });
        expect(progressBar).toBeDefined();
        expect(progressBar.getAttribute("aria-valuenow")).toBe("50");
      });
    });

    it("download completion updates button to 'Model Ready' state", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      // Start download
      const tdtCard = screen.getByText("Batch (TDT)").closest(".transcription-settings__model-card");
      const downloadButton = tdtCard?.querySelector("button");
      await userEvent.click(downloadButton!);

      // Emit completion event
      emitEvent("model_download_completed", {
        modelType: "tdt",
        modelPath: "/path/to/model",
      });

      await waitFor(() => {
        expect(screen.getAllByText("Model Ready").length).toBeGreaterThan(0);
      });
    });

    it("error state displays error message below button", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "download_model") {
          return Promise.reject(new Error("Download failed: Network error"));
        }
        if (cmd === "check_parakeet_model_status") {
          return Promise.resolve(false);
        }
          return Promise.resolve();
      });

      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      const tdtCard = screen.getByText("Batch (TDT)").closest(".transcription-settings__model-card");
      const downloadButton = tdtCard?.querySelector("button");
      await userEvent.click(downloadButton!);

      await waitFor(() => {
        expect(screen.getByText("Download failed: Network error")).toBeDefined();
      });
    });

    it("retry button appears after download error", async () => {
      mockInvoke.mockImplementation((cmd: string) => {
        if (cmd === "download_model") {
          return Promise.reject(new Error("Network error"));
        }
        if (cmd === "check_parakeet_model_status") {
          return Promise.resolve(false);
        }
          return Promise.resolve();
      });

      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      const tdtCard = screen.getByText("Batch (TDT)").closest(".transcription-settings__model-card");
      const downloadButton = tdtCard?.querySelector("button");
      await userEvent.click(downloadButton!);

      await waitFor(() => {
        expect(screen.getByText("Retry Download")).toBeDefined();
      });
    });
  });

  describe("Model status check", () => {
    it("checks model status on component mount via check_parakeet_model_status command", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith("check_parakeet_model_status", { modelType: "tdt" });
      });
    });

    it("displays 'Model Ready' when model is already available", async () => {
      mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
        if (cmd === "check_parakeet_model_status") {
          return Promise.resolve(true);
        }
          return Promise.resolve();
      });

      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Model Ready")).toBeDefined();
      });
    });
  });

  describe("Accessibility", () => {
    it("has proper ARIA labels for model download controls", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      expect(screen.getByLabelText("Click to download Batch (TDT) model")).toBeDefined();
    });

    it("has proper role and aria-label for the settings region", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByRole("region", { name: "Transcription settings" })).toBeDefined();
      });
    });

    it("aria-busy is set during download", async () => {
      render(<TranscriptionSettings />);

      await waitFor(() => {
        expect(screen.getByText("Batch (TDT)")).toBeDefined();
      });

      const tdtCard = screen.getByText("Batch (TDT)").closest(".transcription-settings__model-card");
      const downloadButton = tdtCard?.querySelector("button");
      await userEvent.click(downloadButton!);

      expect(downloadButton?.getAttribute("aria-busy")).toBe("true");
    });
  });
});
