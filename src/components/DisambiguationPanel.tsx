import { useEffect, useRef, useCallback, useState } from "react";
import { useDisambiguation, CommandCandidate } from "../hooks/useDisambiguation";
import "./DisambiguationPanel.css";

export interface DisambiguationPanelProps {
  className?: string;
  /** Auto-dismiss timeout in ms (default: 5000) */
  timeout?: number;
}

/**
 * Panel to display and select from ambiguous command matches.
 * Supports keyboard navigation and auto-dismisses after timeout.
 */
export function DisambiguationPanel({
  className = "",
  timeout = 5000,
}: DisambiguationPanelProps) {
  const { isAmbiguous, transcription, candidates, executeCommand, dismiss } =
    useDisambiguation();
  const [selectedIndex, setSelectedIndex] = useState(0);
  const panelRef = useRef<HTMLDivElement>(null);

  // Reset selection when candidates change
  useEffect(() => {
    setSelectedIndex(0);
  }, [candidates]);

  // Auto-dismiss after timeout
  useEffect(() => {
    if (!isAmbiguous) return;

    const timer = setTimeout(() => {
      dismiss();
    }, timeout);

    return () => clearTimeout(timer);
  }, [isAmbiguous, timeout, dismiss]);

  // Handle keyboard navigation
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!isAmbiguous || candidates.length === 0) return;

      switch (event.key) {
        case "ArrowDown":
          event.preventDefault();
          setSelectedIndex((prev) => (prev + 1) % candidates.length);
          break;
        case "ArrowUp":
          event.preventDefault();
          setSelectedIndex(
            (prev) => (prev - 1 + candidates.length) % candidates.length
          );
          break;
        case "Enter":
          event.preventDefault();
          executeCommand(candidates[selectedIndex].id);
          break;
        case "Escape":
          event.preventDefault();
          dismiss();
          break;
      }
    },
    [isAmbiguous, candidates, selectedIndex, executeCommand, dismiss]
  );

  // Register keyboard listener
  useEffect(() => {
    if (!isAmbiguous) return;

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isAmbiguous, handleKeyDown]);

  // Focus the panel when it appears
  useEffect(() => {
    if (isAmbiguous && panelRef.current) {
      panelRef.current.focus();
    }
  }, [isAmbiguous]);

  const handleCandidateClick = (candidate: CommandCandidate) => {
    executeCommand(candidate.id);
  };

  if (!isAmbiguous || candidates.length === 0) {
    return null;
  }

  return (
    <div
      ref={panelRef}
      className={`disambiguation-panel ${className}`.trim()}
      role="dialog"
      aria-label="Select command"
      tabIndex={-1}
    >
      <div className="disambiguation-panel__header">
        <span className="disambiguation-panel__title">
          Which command did you mean?
        </span>
        <span className="disambiguation-panel__transcription">
          "{transcription}"
        </span>
      </div>
      <ul
        className="disambiguation-panel__candidates"
        role="listbox"
        aria-activedescendant={`candidate-${selectedIndex}`}
      >
        {candidates.map((candidate, index) => (
          <li
            key={candidate.id}
            id={`candidate-${index}`}
            className={`disambiguation-panel__candidate ${
              index === selectedIndex
                ? "disambiguation-panel__candidate--selected"
                : ""
            }`}
            role="option"
            aria-selected={index === selectedIndex}
            onClick={() => handleCandidateClick(candidate)}
          >
            <span className="disambiguation-panel__trigger">
              {candidate.trigger}
            </span>
            <span className="disambiguation-panel__confidence">
              {Math.round(candidate.confidence * 100)}%
            </span>
          </li>
        ))}
      </ul>
      <div className="disambiguation-panel__footer">
        <span className="disambiguation-panel__hint">
          Press Enter to select, Escape to cancel
        </span>
        <button
          className="disambiguation-panel__cancel"
          onClick={dismiss}
          aria-label="Cancel"
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
