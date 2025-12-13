import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import { DisambiguationPanel } from "./DisambiguationPanel";
import * as useDisambiguationModule from "../hooks/useDisambiguation";

vi.mock("../hooks/useDisambiguation");

const mockUseDisambiguation = vi.mocked(
  useDisambiguationModule.useDisambiguation
);

describe("DisambiguationPanel", () => {
  const mockExecuteCommand = vi.fn();
  const mockDismiss = vi.fn();

  const defaultMock: useDisambiguationModule.UseDisambiguationResult = {
    isAmbiguous: false,
    transcription: null,
    candidates: [],
    executeCommand: mockExecuteCommand,
    dismiss: mockDismiss,
  };

  const candidatesFixture = [
    { id: "1", trigger: "open slack", confidence: 0.85 },
    { id: "2", trigger: "open safari", confidence: 0.82 },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    mockUseDisambiguation.mockReturnValue(defaultMock);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("is hidden when not ambiguous", () => {
    const { container } = render(<DisambiguationPanel />);
    expect(container.firstChild).toBeNull();
  });

  it("is hidden when ambiguous but no candidates", () => {
    mockUseDisambiguation.mockReturnValue({
      ...defaultMock,
      isAmbiguous: true,
      transcription: "open",
      candidates: [],
    });

    const { container } = render(<DisambiguationPanel />);
    expect(container.firstChild).toBeNull();
  });

  it("displays 2 candidate commands when ambiguous event received", () => {
    mockUseDisambiguation.mockReturnValue({
      ...defaultMock,
      isAmbiguous: true,
      transcription: "open app",
      candidates: candidatesFixture,
    });

    render(<DisambiguationPanel />);

    expect(screen.getByText("Which command did you mean?")).toBeDefined();
    expect(screen.getByText('"open app"')).toBeDefined();
    expect(screen.getByText("open slack")).toBeDefined();
    expect(screen.getByText("open safari")).toBeDefined();
    expect(screen.getByText("85%")).toBeDefined();
    expect(screen.getByText("82%")).toBeDefined();
  });

  it("click on candidate triggers execution", () => {
    mockUseDisambiguation.mockReturnValue({
      ...defaultMock,
      isAmbiguous: true,
      transcription: "open app",
      candidates: candidatesFixture,
    });

    render(<DisambiguationPanel />);

    const slackOption = screen.getByText("open slack");
    fireEvent.click(slackOption);

    expect(mockExecuteCommand).toHaveBeenCalledWith("1");
  });

  it("keyboard navigation with arrow down selects next candidate", () => {
    mockUseDisambiguation.mockReturnValue({
      ...defaultMock,
      isAmbiguous: true,
      transcription: "open app",
      candidates: candidatesFixture,
    });

    render(<DisambiguationPanel />);

    // First candidate should be selected by default
    const options = screen.getAllByRole("option");
    expect(options[0].getAttribute("aria-selected")).toBe("true");
    expect(options[1].getAttribute("aria-selected")).toBe("false");

    // Press down arrow
    act(() => {
      fireEvent.keyDown(document, { key: "ArrowDown" });
    });

    // Second candidate should now be selected
    const updatedOptions = screen.getAllByRole("option");
    expect(updatedOptions[0].getAttribute("aria-selected")).toBe("false");
    expect(updatedOptions[1].getAttribute("aria-selected")).toBe("true");
  });

  it("keyboard navigation with arrow up selects previous candidate", () => {
    mockUseDisambiguation.mockReturnValue({
      ...defaultMock,
      isAmbiguous: true,
      transcription: "open app",
      candidates: candidatesFixture,
    });

    render(<DisambiguationPanel />);

    // Press down to select second, then up to go back
    act(() => {
      fireEvent.keyDown(document, { key: "ArrowDown" });
    });
    act(() => {
      fireEvent.keyDown(document, { key: "ArrowUp" });
    });

    const options = screen.getAllByRole("option");
    expect(options[0].getAttribute("aria-selected")).toBe("true");
  });

  it("keyboard navigation wraps around", () => {
    mockUseDisambiguation.mockReturnValue({
      ...defaultMock,
      isAmbiguous: true,
      transcription: "open app",
      candidates: candidatesFixture,
    });

    render(<DisambiguationPanel />);

    // Press up from first item should wrap to last
    act(() => {
      fireEvent.keyDown(document, { key: "ArrowUp" });
    });

    const options = screen.getAllByRole("option");
    expect(options[1].getAttribute("aria-selected")).toBe("true");
  });

  it("enter key triggers execution of selected candidate", () => {
    mockUseDisambiguation.mockReturnValue({
      ...defaultMock,
      isAmbiguous: true,
      transcription: "open app",
      candidates: candidatesFixture,
    });

    render(<DisambiguationPanel />);

    // Press down to select second candidate
    act(() => {
      fireEvent.keyDown(document, { key: "ArrowDown" });
    });

    // Press enter
    act(() => {
      fireEvent.keyDown(document, { key: "Enter" });
    });

    expect(mockExecuteCommand).toHaveBeenCalledWith("2");
  });

  it("escape key dismisses without execution", () => {
    mockUseDisambiguation.mockReturnValue({
      ...defaultMock,
      isAmbiguous: true,
      transcription: "open app",
      candidates: candidatesFixture,
    });

    render(<DisambiguationPanel />);

    act(() => {
      fireEvent.keyDown(document, { key: "Escape" });
    });

    expect(mockDismiss).toHaveBeenCalled();
    expect(mockExecuteCommand).not.toHaveBeenCalled();
  });

  it("5-second timeout auto-dismisses", () => {
    mockUseDisambiguation.mockReturnValue({
      ...defaultMock,
      isAmbiguous: true,
      transcription: "open app",
      candidates: candidatesFixture,
    });

    render(<DisambiguationPanel timeout={5000} />);

    expect(mockDismiss).not.toHaveBeenCalled();

    // Advance time by 5 seconds
    act(() => {
      vi.advanceTimersByTime(5000);
    });

    expect(mockDismiss).toHaveBeenCalled();
  });

  it("cancel button dismisses without execution", () => {
    mockUseDisambiguation.mockReturnValue({
      ...defaultMock,
      isAmbiguous: true,
      transcription: "open app",
      candidates: candidatesFixture,
    });

    render(<DisambiguationPanel />);

    const cancelButton = screen.getByRole("button", { name: "Cancel" });
    fireEvent.click(cancelButton);

    expect(mockDismiss).toHaveBeenCalled();
    expect(mockExecuteCommand).not.toHaveBeenCalled();
  });

  it("applies custom className", () => {
    mockUseDisambiguation.mockReturnValue({
      ...defaultMock,
      isAmbiguous: true,
      transcription: "open app",
      candidates: candidatesFixture,
    });

    render(<DisambiguationPanel className="custom-class" />);

    const panel = screen.getByRole("dialog");
    expect(panel.className).toContain("custom-class");
  });

  it("has correct accessibility attributes", () => {
    mockUseDisambiguation.mockReturnValue({
      ...defaultMock,
      isAmbiguous: true,
      transcription: "open app",
      candidates: candidatesFixture,
    });

    render(<DisambiguationPanel />);

    const dialog = screen.getByRole("dialog");
    expect(dialog.getAttribute("aria-label")).toBe("Select command");

    const listbox = screen.getByRole("listbox");
    expect(listbox).toBeDefined();
  });
});
