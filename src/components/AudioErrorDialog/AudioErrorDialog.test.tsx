import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AudioErrorDialog } from "./AudioErrorDialog";
import { AudioDeviceError } from "../../types/audio";

describe("AudioErrorDialog", () => {
  const defaultProps = {
    error: null as AudioDeviceError | null,
    onRetry: vi.fn(),
    onSelectDevice: vi.fn(),
    onDismiss: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders nothing when error is null", () => {
    const { container } = render(<AudioErrorDialog {...defaultProps} />);
    expect(container.firstChild).toBeNull();
  });

  describe("deviceNotFound error", () => {
    const error: AudioDeviceError = {
      type: "deviceNotFound",
      deviceName: "USB Microphone",
    };

    it("shows correct title and message", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      expect(screen.getByText("Microphone Not Found")).toBeDefined();
      expect(
        screen.getByText(
          'The selected microphone "USB Microphone" is not connected.'
        )
      ).toBeDefined();
    });

    it("shows Select Device and Try Again buttons", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      expect(screen.getByText("Select Device")).toBeDefined();
      expect(screen.getByText("Try Again")).toBeDefined();
    });

    it("calls onSelectDevice when Select Device is clicked", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      fireEvent.click(screen.getByText("Select Device"));
      expect(defaultProps.onSelectDevice).toHaveBeenCalledTimes(1);
    });

    it("calls onRetry when Try Again is clicked", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      fireEvent.click(screen.getByText("Try Again"));
      expect(defaultProps.onRetry).toHaveBeenCalledTimes(1);
    });
  });

  describe("noDevicesAvailable error", () => {
    const error: AudioDeviceError = { type: "noDevicesAvailable" };

    it("shows correct title", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      expect(screen.getByText("No Microphone Detected")).toBeDefined();
    });

    it("shows only Try Again button (no Select Device)", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      expect(screen.getByText("Try Again")).toBeDefined();
      expect(screen.queryByText("Select Device")).toBeNull();
    });
  });

  describe("permissionDenied error", () => {
    const error: AudioDeviceError = { type: "permissionDenied" };

    it("shows correct title", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      expect(screen.getByText("Microphone Access Required")).toBeDefined();
    });

    it("shows Open Settings and Try Again buttons", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      expect(screen.getByText("Open Settings")).toBeDefined();
      expect(screen.getByText("Try Again")).toBeDefined();
    });
  });

  describe("deviceDisconnected error", () => {
    const error: AudioDeviceError = { type: "deviceDisconnected" };

    it("shows correct title", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      expect(screen.getByText("Microphone Disconnected")).toBeDefined();
    });

    it("shows Select Device and Try Again buttons", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      expect(screen.getByText("Select Device")).toBeDefined();
      expect(screen.getByText("Try Again")).toBeDefined();
    });
  });

  describe("captureError error", () => {
    const error: AudioDeviceError = {
      type: "captureError",
      message: "Stream initialization failed",
    };

    it("shows correct title", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      expect(screen.getByText("Recording Error")).toBeDefined();
    });

    it("shows custom error message", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      expect(
        screen.getByText("Stream initialization failed")
      ).toBeDefined();
    });

    it("shows only Try Again button", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      expect(screen.getByText("Try Again")).toBeDefined();
      expect(screen.queryByText("Select Device")).toBeNull();
    });
  });

  describe("dismiss behavior", () => {
    const error: AudioDeviceError = { type: "noDevicesAvailable" };

    it("calls onDismiss when Dismiss button is clicked", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      fireEvent.click(screen.getByText("Dismiss"));
      expect(defaultProps.onDismiss).toHaveBeenCalledTimes(1);
    });

    it("calls onDismiss when Escape key is pressed", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      const overlay = screen.getByRole("dialog");
      fireEvent.keyDown(overlay, { key: "Escape" });
      expect(defaultProps.onDismiss).toHaveBeenCalledTimes(1);
    });
  });

  describe("accessibility", () => {
    const error: AudioDeviceError = { type: "noDevicesAvailable" };

    it("has dialog role and aria-modal", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      const dialog = screen.getByRole("dialog");
      expect(dialog.getAttribute("aria-modal")).toBe("true");
    });

    it("has aria-labelledby pointing to title", () => {
      render(<AudioErrorDialog {...defaultProps} error={error} />);

      const dialog = screen.getByRole("dialog");
      expect(dialog.getAttribute("aria-labelledby")).toBe("audio-error-title");
      expect(screen.getByText("No Microphone Detected").id).toBe(
        "audio-error-title"
      );
    });
  });

  it("applies custom className", () => {
    const error: AudioDeviceError = { type: "noDevicesAvailable" };
    render(
      <AudioErrorDialog
        {...defaultProps}
        error={error}
        className="custom-class"
      />
    );

    expect(screen.getByRole("dialog").className).toContain("custom-class");
  });
});
