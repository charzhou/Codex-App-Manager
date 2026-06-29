import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";

import { CodexConfig } from "./CodexConfig";

const {
  getCodexCliPresetPreview,
  applyCodexCliPreset,
  openCodexHome,
} = vi.hoisted(() => ({
  getCodexCliPresetPreview: vi.fn(),
  applyCodexCliPreset: vi.fn(),
  openCodexHome: vi.fn(),
}));

vi.mock("../../services/managerApi", () => ({
  managerApi: {
    getCodexCliPresetPreview,
    applyCodexCliPreset,
    openCodexHome,
  },
  errorMessage: (cause: unknown) => (cause instanceof Error ? cause.message : String(cause)),
}));

vi.mock("../i18n", () => ({
  useI18n: () => ({
    t: (key: string) => key,
  }),
}));

vi.mock("../components", () => ({
  NavBar: ({ title }: { title: string }) => <div>{title}</div>,
  Ring: () => <div>ring</div>,
}));

vi.mock("../icons", () => ({
  Icon: () => <span>icon</span>,
}));

describe("CodexConfig", () => {
  beforeEach(() => {
    getCodexCliPresetPreview.mockReset();
    applyCodexCliPreset.mockReset();
    openCodexHome.mockReset();
    getCodexCliPresetPreview.mockResolvedValue({
      codexHomePath: "~/.codex",
      configTomlPath: "~/.codex/config.toml",
      authJsonPath: "~/.codex/auth.json",
      backupDirPath: "~/.codex/backups",
      configSnippet: 'model_provider = "OpenAI"',
      authJsonPreview: '{\n  "OPENAI_API_KEY": "sk-***"\n}',
      warning: "warning",
    });
  });

  it("disables apply until an API key is entered", async () => {
    render(<CodexConfig onBack={() => undefined} />);
    const button = await screen.findByRole("button", { name: "config.apply" });
    expect(button).toBeDisabled();
  });

  it("keeps preview and log collapsed by default and expands them on click", async () => {
    render(<CodexConfig onBack={() => undefined} />);
    await screen.findByText("config.pathsHeader");
    expect(screen.queryByText('model_provider = "OpenAI"')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /config.previewHeader/i }));
    expect(await screen.findByText('model_provider = "OpenAI"')).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /config.logHeader/i }));
    expect(await screen.findByText("config.noLogsYet")).toBeInTheDocument();
  });

  it("shows success state and logs after apply", async () => {
    applyCodexCliPreset.mockResolvedValue({
      codexHomePath: "~/.codex",
      configTomlPath: "~/.codex/config.toml",
      authJsonPath: "~/.codex/auth.json",
      backupDirPath: "~/.codex/backups",
      backups: [
        {
          targetPath: "~/.codex/config.toml",
          backupPath: "~/.codex/backups/config.toml.20260629-190000.bak",
        },
      ],
      logs: [
        {
          step: "done",
          message: "Applied Codex CLI preset.",
          status: "success",
          detail: null,
        },
      ],
      warning: null,
    });

    render(<CodexConfig onBack={() => undefined} />);
    const input = await screen.findByPlaceholderText("config.apiKeyPlaceholder");
    fireEvent.change(input, { target: { value: "sk-test" } });
    fireEvent.click(screen.getByRole("button", { name: "config.apply" }));

    expect(await screen.findByText("config.applySuccess")).toBeInTheDocument();
    expect(await screen.findByText("Applied Codex CLI preset.")).toBeInTheDocument();
    expect(
      await screen.findByText("~/.codex/backups/config.toml.20260629-190000.bak"),
    ).toBeInTheDocument();
  });

  it("shows failure when apply rejects", async () => {
    applyCodexCliPreset.mockRejectedValue(new Error("write auth.json failed"));

    render(<CodexConfig onBack={() => undefined} />);
    const input = await screen.findByPlaceholderText("config.apiKeyPlaceholder");
    fireEvent.change(input, { target: { value: "sk-test" } });
    fireEvent.click(screen.getByRole("button", { name: "config.apply" }));

    expect(await screen.findByText("write auth.json failed")).toBeInTheDocument();
  });
});
