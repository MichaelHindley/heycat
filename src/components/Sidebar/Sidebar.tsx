import { useState, useEffect } from "react";
import { RecordingsList } from "../RecordingsView";
import { CommandSettings } from "../CommandSettings";
import { TranscriptionSettings } from "../TranscriptionSettings";
import { ListeningSettings } from "../ListeningSettings";
import "./Sidebar.css";

export type SidebarTab = "history" | "commands" | "transcription" | "listening";

export interface SidebarProps {
  className?: string;
  defaultTab?: SidebarTab;
  /** Controlled active tab (optional) */
  activeTab?: SidebarTab;
  /** Callback when tab changes (required if activeTab is provided) */
  onTabChange?: (tab: SidebarTab) => void;
}

export function Sidebar({
  className = "",
  defaultTab = "history",
  activeTab: controlledTab,
  onTabChange,
}: SidebarProps) {
  const [internalTab, setInternalTab] = useState<SidebarTab>(defaultTab);

  // Use controlled tab if provided, otherwise use internal state
  const activeTab = controlledTab ?? internalTab;
  const setActiveTab = onTabChange ?? setInternalTab;

  // Sync internal state if controlled tab changes
  useEffect(() => {
    if (controlledTab !== undefined) {
      setInternalTab(controlledTab);
    }
  }, [controlledTab]);

  return (
    <aside className={`sidebar ${className}`.trim()} role="complementary">
      <nav className="sidebar__nav" role="tablist" aria-label="Sidebar navigation">
        <button
          className={`sidebar__tab ${activeTab === "history" ? "sidebar__tab--active" : ""}`.trim()}
          role="tab"
          aria-selected={activeTab === "history"}
          aria-controls="sidebar-panel-history"
          onClick={() => setActiveTab("history")}
          type="button"
        >
          History
        </button>
        <button
          className={`sidebar__tab ${activeTab === "commands" ? "sidebar__tab--active" : ""}`.trim()}
          role="tab"
          aria-selected={activeTab === "commands"}
          aria-controls="sidebar-panel-commands"
          onClick={() => setActiveTab("commands")}
          type="button"
        >
          Commands
        </button>
        <button
          className={`sidebar__tab ${activeTab === "transcription" ? "sidebar__tab--active" : ""}`.trim()}
          role="tab"
          aria-selected={activeTab === "transcription"}
          aria-controls="sidebar-panel-transcription"
          onClick={() => setActiveTab("transcription")}
          type="button"
        >
          Transcription
        </button>
        <button
          className={`sidebar__tab ${activeTab === "listening" ? "sidebar__tab--active" : ""}`.trim()}
          role="tab"
          aria-selected={activeTab === "listening"}
          aria-controls="sidebar-panel-listening"
          onClick={() => setActiveTab("listening")}
          type="button"
        >
          Listening
        </button>
      </nav>
      <div
        id={`sidebar-panel-${activeTab}`}
        className="sidebar__panel"
        role="tabpanel"
        aria-labelledby={`sidebar-tab-${activeTab}`}
      >
        {activeTab === "history" && <RecordingsList />}
        {activeTab === "commands" && <CommandSettings />}
        {activeTab === "transcription" && <TranscriptionSettings />}
        {activeTab === "listening" && <ListeningSettings />}
      </div>
    </aside>
  );
}
