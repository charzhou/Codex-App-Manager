import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { App } from "./App";

vi.mock("./theme", () => ({
  ThemeProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

vi.mock("./i18n", () => ({
  I18nProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

vi.mock("./viewTransition", () => ({
  withViewTransition: (fn: () => void) => fn(),
}));

vi.mock("./components", () => ({
  QuitConfirm: () => null,
}));

vi.mock("./views/Home", () => ({
  Home: ({
    onOpenSettings,
    onOpenConfig,
  }: {
    onOpenSettings: () => void;
    onOpenConfig: () => void;
  }) => (
    <div>
      <button onClick={onOpenSettings}>open-settings</button>
      <button onClick={onOpenConfig}>open-config</button>
    </div>
  ),
}));

vi.mock("./views/Settings", () => ({
  Settings: ({
    onBack,
    onOpenConfig,
  }: {
    onBack: () => void;
    onOpenConfig: () => void;
  }) => (
    <div>
      <button onClick={onBack}>settings-back</button>
      <button onClick={onOpenConfig}>settings-config</button>
    </div>
  ),
}));

vi.mock("./views/About", () => ({
  About: () => <div>about-view</div>,
}));

vi.mock("./views/Uninstall", () => ({
  Uninstall: () => <div>uninstall-view</div>,
}));

vi.mock("./views/CodexConfig", () => ({
  CodexConfig: ({ onBack }: { onBack: () => void }) => (
    <div>
      <div>config-view</div>
      <button onClick={onBack}>config-back</button>
    </div>
  ),
}));

describe("App", () => {
  it("opens Codex config directly from home", async () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "open-config" }));

    expect(await screen.findByText("config-view")).toBeInTheDocument();
  });
});
