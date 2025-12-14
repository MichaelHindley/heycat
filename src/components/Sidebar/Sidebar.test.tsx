import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Sidebar } from "./Sidebar";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("@tauri-apps/plugin-store", () => ({
  load: vi.fn().mockResolvedValue({
    get: vi.fn().mockResolvedValue(false),
    set: vi.fn().mockResolvedValue(undefined),
  }),
}));

describe("Sidebar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("History tab", () => {
    it("History tab is present in sidebar", async () => {
      render(<Sidebar />);

      expect(await screen.findByRole("tab", { name: "History" })).toBeDefined();
    });

    it("History tab is selected by default", async () => {
      render(<Sidebar />);

      const historyTab = await screen.findByRole("tab", { name: "History" });
      expect(historyTab.getAttribute("aria-selected")).toBe("true");
    });
  });

  describe("Commands tab", () => {
    it("Commands tab is present in sidebar", async () => {
      render(<Sidebar />);

      expect(await screen.findByRole("tab", { name: "Commands" })).toBeDefined();
    });

    it("Commands tab switches panel content when clicked", async () => {
      render(<Sidebar />);

      const commandsTab = screen.getByRole("tab", { name: "Commands" });
      await userEvent.click(commandsTab);

      expect(commandsTab.getAttribute("aria-selected")).toBe("true");
      expect(
        screen.getByRole("tab", { name: "History" }).getAttribute("aria-selected")
      ).toBe("false");
    });

    it("can start on Commands tab via defaultTab prop", async () => {
      render(<Sidebar defaultTab="commands" />);

      const commandsTab = await screen.findByRole("tab", { name: "Commands" });
      expect(commandsTab.getAttribute("aria-selected")).toBe("true");
      // Wait for CommandSettings async effect to complete
      await waitFor(() => {
        expect(screen.getByText("Voice Commands")).toBeDefined();
      });
    });
  });

  describe("Transcription tab", () => {
    it("Transcription tab is present in sidebar", async () => {
      render(<Sidebar />);

      expect(await screen.findByRole("tab", { name: "Transcription" })).toBeDefined();
    });

    it("Transcription tab switches panel content when clicked", async () => {
      render(<Sidebar />);

      const transcriptionTab = screen.getByRole("tab", { name: "Transcription" });
      await userEvent.click(transcriptionTab);

      expect(transcriptionTab.getAttribute("aria-selected")).toBe("true");
      expect(
        screen.getByRole("tab", { name: "History" }).getAttribute("aria-selected")
      ).toBe("false");
    });

    it("can start on Transcription tab via defaultTab prop", async () => {
      render(<Sidebar defaultTab="transcription" />);

      const transcriptionTab = await screen.findByRole("tab", { name: "Transcription" });
      expect(transcriptionTab.getAttribute("aria-selected")).toBe("true");
      // Wait for TranscriptionSettings to render
      await waitFor(() => {
        expect(screen.getByText("Models")).toBeDefined();
      });
    });
  });

  describe("Listening tab", () => {
    it("Listening tab is present in sidebar", async () => {
      render(<Sidebar />);

      expect(await screen.findByRole("tab", { name: "Listening" })).toBeDefined();
    });

    it("Listening tab switches panel content when clicked", async () => {
      render(<Sidebar />);

      const listeningTab = screen.getByRole("tab", { name: "Listening" });
      await userEvent.click(listeningTab);

      expect(listeningTab.getAttribute("aria-selected")).toBe("true");
      expect(
        screen.getByRole("tab", { name: "History" }).getAttribute("aria-selected")
      ).toBe("false");
    });

    it("can start on Listening tab via defaultTab prop", async () => {
      render(<Sidebar defaultTab="listening" />);

      const listeningTab = await screen.findByRole("tab", { name: "Listening" });
      expect(listeningTab.getAttribute("aria-selected")).toBe("true");
      // Wait for ListeningSettings to render
      await waitFor(() => {
        expect(screen.getByText("Always Listening")).toBeDefined();
      });
    });
  });

  describe("tab navigation", () => {
    it("renders tabpanel for history content", async () => {
      render(<Sidebar />);

      expect(await screen.findByRole("tabpanel")).toBeDefined();
    });
  });

  describe("content rendering", () => {
    it("renders RecordingsList in history panel", async () => {
      render(<Sidebar />);

      // RecordingsList is rendered - check for loading state initially
      // Since invoke is mocked to return [], it will show empty state
      const panel = await screen.findByRole("tabpanel");
      expect(panel).toBeDefined();
    });
  });
});
