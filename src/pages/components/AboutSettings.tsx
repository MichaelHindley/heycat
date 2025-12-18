import { ExternalLink, Github, FileText, Bug } from "lucide-react";
import { Card, CardContent } from "../../components/ui";

export interface AboutSettingsProps {
  className?: string;
}

// App version - would typically come from package.json or build config
const APP_VERSION = "1.0.0";

export function AboutSettings({ className = "" }: AboutSettingsProps) {
  const handleOpenLink = (url: string) => {
    window.open(url, "_blank", "noopener,noreferrer");
  };

  return (
    <div className={`space-y-6 ${className}`.trim()}>
      {/* App Info Section */}
      <section>
        <Card>
          <CardContent className="text-center py-8 space-y-4">
            {/* App Logo/Icon would go here */}
            <div className="w-16 h-16 mx-auto bg-heycat-orange/20 rounded-2xl flex items-center justify-center">
              <span className="text-3xl">🐱</span>
            </div>

            <div>
              <h2 className="text-xl font-semibold text-text-primary">HeyCat</h2>
              <p className="text-sm text-text-secondary">Version {APP_VERSION}</p>
            </div>

            <p className="text-sm text-text-secondary max-w-md mx-auto">
              A voice-controlled assistant for your Mac. Say "Hey Cat" to start
              recording and let HeyCat transcribe your voice into text or execute
              custom commands.
            </p>
          </CardContent>
        </Card>
      </section>

      {/* Links Section */}
      <section>
        <h2 className="text-xs font-semibold text-text-secondary uppercase tracking-wider mb-4">
          Resources
        </h2>
        <Card>
          <CardContent className="space-y-1 p-2">
            <LinkButton
              icon={<Github className="h-4 w-4" />}
              label="GitHub Repository"
              description="View source code and contribute"
              onClick={() => handleOpenLink("https://github.com/heycat-app/heycat")}
            />
            <LinkButton
              icon={<FileText className="h-4 w-4" />}
              label="Documentation"
              description="Learn how to use HeyCat"
              onClick={() => handleOpenLink("https://heycat.app/docs")}
            />
            <LinkButton
              icon={<Bug className="h-4 w-4" />}
              label="Report an Issue"
              description="Found a bug? Let us know"
              onClick={() =>
                handleOpenLink("https://github.com/heycat-app/heycat/issues/new")
              }
            />
          </CardContent>
        </Card>
      </section>

      {/* Credits Section */}
      <section>
        <h2 className="text-xs font-semibold text-text-secondary uppercase tracking-wider mb-4">
          Acknowledgments
        </h2>
        <Card>
          <CardContent className="space-y-3">
            <p className="text-sm text-text-secondary">
              HeyCat is built with these amazing technologies:
            </p>
            <ul className="text-sm text-text-secondary space-y-1">
              <li>
                <span className="text-text-primary font-medium">Tauri</span> -
                Cross-platform desktop framework
              </li>
              <li>
                <span className="text-text-primary font-medium">React</span> -
                UI framework
              </li>
              <li>
                <span className="text-text-primary font-medium">
                  Parakeet TDT
                </span>{" "}
                - Speech-to-text model
              </li>
              <li>
                <span className="text-text-primary font-medium">Silero VAD</span>{" "}
                - Voice activity detection
              </li>
            </ul>
            <p className="text-xs text-text-secondary pt-2 border-t border-border">
              Made with ❤️ for Mac users who prefer talking to typing.
            </p>
          </CardContent>
        </Card>
      </section>
    </div>
  );
}

interface LinkButtonProps {
  icon: React.ReactNode;
  label: string;
  description: string;
  onClick: () => void;
}

function LinkButton({ icon, label, description, onClick }: LinkButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="
        w-full flex items-center gap-3 p-3 rounded-lg
        text-left
        hover:bg-surface
        transition-colors duration-fast
        focus:outline-none focus-visible:ring-2 focus-visible:ring-heycat-teal
      "
    >
      <div className="text-text-secondary">{icon}</div>
      <div className="flex-1 min-w-0">
        <div className="text-sm font-medium text-text-primary">{label}</div>
        <div className="text-xs text-text-secondary">{description}</div>
      </div>
      <ExternalLink className="h-4 w-4 text-text-secondary" />
    </button>
  );
}
