import { useEffect, useMemo, useState } from "react";

import { errorLogs, errorMessage, managerApi } from "../../services/managerApi";
import type {
  CodexCliPresetApplyResult,
  CodexCliPresetPreview,
  CodexConfigLogStep,
} from "../../shared/types";
import { Icon } from "../icons";
import { useI18n } from "../i18n";
import { NavBar, Ring } from "../components";

function maskApiKey(value: string): string {
  if (!value) return "";
  if (value.length <= 8) return "*".repeat(value.length);
  return `${value.slice(0, 4)}${"*".repeat(Math.max(4, value.length - 8))}${value.slice(-4)}`;
}

export function CodexConfig({ onBack }: { onBack: () => void }) {
  const { t } = useI18n();
  const [apiKey, setApiKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [preview, setPreview] = useState<CodexCliPresetPreview | null>(null);
  const [previewOpen, setPreviewOpen] = useState(false);
  const [logOpen, setLogOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<CodexCliPresetApplyResult | null>(null);
  const [logs, setLogs] = useState<CodexConfigLogStep[]>([]);

  useEffect(() => {
    void managerApi
      .getCodexCliPresetPreview()
      .then(setPreview)
      .catch((cause) => setError(errorMessage(cause)));
  }, []);

  const authPreview = useMemo(() => {
    if (!preview) return "";
    if (!apiKey.trim()) return preview.authJsonPreview;
    return JSON.stringify({ OPENAI_API_KEY: maskApiKey(apiKey.trim()) }, null, 2);
  }, [apiKey, preview]);

  const apply = async () => {
    setBusy(true);
    setError(null);
    setResult(null);
    setLogs([]);
    try {
      const next = await managerApi.applyCodexCliPreset(apiKey);
      setResult(next);
      setLogs(next.logs);
      setLogOpen(true);
    } catch (cause) {
      setError(errorMessage(cause));
      setLogs(errorLogs(cause) ?? []);
      setLogOpen(true);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="pop">
      <NavBar title={t("nav.config")} onBack={onBack} />
      <div className="scroll view">
        <section className="hero" style={{ marginTop: 16 }}>
          <Ring icon="sliders" variant="muted" />
          <div className="headline" style={{ fontSize: 18 }}>
            {t("config.title")}
          </div>
          <div className="desc">{t("config.descV2")}</div>
        </section>

        {error ? (
          <div className="banner err">
            <Icon name="alert" />
            <span>{error}</span>
          </div>
        ) : null}

        {result ? (
          <div className="banner info">
            <Icon name="check" />
            <span>{t("config.applySuccess")}</span>
          </div>
        ) : null}

        <div className="group">
          <div className="group-h">{t("config.apiKeyHeader")}</div>
          <div className="list">
            <div className="row" style={{ display: "block" }}>
              <div className="rtitle" style={{ marginBottom: 8 }}>
                {t("config.apiKeyLabel")}
              </div>
              <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                <input
                  className="input mono"
                  type={showKey ? "text" : "password"}
                  value={apiKey}
                  placeholder={t("config.apiKeyPlaceholder")}
                  onChange={(e) => setApiKey(e.target.value)}
                  style={{ flex: 1 }}
                />
                <button className="mini-action" onClick={() => setShowKey((v) => !v)}>
                  {showKey ? t("config.hideKey") : t("config.showKey")}
                </button>
              </div>
            </div>
          </div>
        </div>

        {preview ? (
          <div className="group">
            <div className="group-h">{t("config.pathsHeader")}</div>
            <div className="list">
              <div className="row">
                <span className="rtext">
                  <span className="rtitle">{t("config.codexHome")}</span>
                  <span className="rsub mono">{preview.codexHomePath}</span>
                </span>
              </div>
              <div className="row">
                <span className="rtext">
                  <span className="rtitle">{t("config.configPath")}</span>
                  <span className="rsub mono">{preview.configTomlPath}</span>
                </span>
              </div>
              <div className="row">
                <span className="rtext">
                  <span className="rtitle">{t("config.authPath")}</span>
                  <span className="rsub mono">{preview.authJsonPath}</span>
                </span>
              </div>
              <div className="row">
                <span className="rtext">
                  <span className="rtitle">{t("config.backupDir")}</span>
                  <span className="rsub mono">{preview.backupDirPath}</span>
                </span>
              </div>
              <div className="row">
                <button className="mini-action" onClick={() => void managerApi.openCodexHome()}>
                  <Icon name="folder" />
                  {t("config.openCodexHome")}
                </button>
              </div>
            </div>
          </div>
        ) : null}

        <div className="group">
          <button className="row" onClick={() => setPreviewOpen((v) => !v)}>
            <span className="rtext">
              <span className="rtitle">{t("config.previewHeader")}</span>
              <span className="rsub">{t("config.previewSub")}</span>
            </span>
            <span className="rval">{previewOpen ? t("config.collapse") : t("config.expand")}</span>
          </button>
          {previewOpen && preview ? (
            <div className="list">
              <div className="row" style={{ display: "block" }}>
                <div className="rtitle">{t("config.previewConfigToml")}</div>
                <pre
                  className="mono"
                  style={{
                    whiteSpace: "pre-wrap",
                    overflowX: "auto",
                    padding: 12,
                    borderRadius: 16,
                    background: "rgba(8, 16, 32, 0.55)",
                    marginTop: 8,
                  }}
                >
                  {preview.configSnippet}
                </pre>
              </div>
              <div className="row" style={{ display: "block" }}>
                <div className="rtitle">{t("config.previewAuthJson")}</div>
                <pre
                  className="mono"
                  style={{
                    whiteSpace: "pre-wrap",
                    overflowX: "auto",
                    padding: 12,
                    borderRadius: 16,
                    background: "rgba(8, 16, 32, 0.55)",
                    marginTop: 8,
                  }}
                >
                  {authPreview}
                </pre>
              </div>
              <div className="banner info">
                <Icon name="info" />
                <span>{preview.warning}</span>
              </div>
            </div>
          ) : null}
        </div>

        <div className="group">
          <button className="row" onClick={() => setLogOpen((v) => !v)}>
            <span className="rtext">
              <span className="rtitle">{t("config.logHeader")}</span>
              <span className="rsub">{t("config.logSub")}</span>
            </span>
            <span className="rval">{logOpen ? t("config.collapse") : t("config.expand")}</span>
          </button>
          {logOpen ? (
            <div className="list">
              {logs.length === 0 ? (
                <div className="row">
                  <span className="rsub">{t("config.noLogsYet")}</span>
                </div>
              ) : (
                logs.map((entry, index) => (
                  <div className="row" key={`${entry.step}-${index}`}>
                    <span className="rtext">
                      <span className="rtitle">{entry.message}</span>
                      <span className="rsub mono">
                        [{entry.status}] {entry.step}
                        {entry.detail ? ` · ${entry.detail}` : ""}
                      </span>
                    </span>
                  </div>
                ))
              )}
            </div>
          ) : null}
        </div>

        {result ? (
          <div className="group">
            <div className="group-h">{t("config.resultHeader")}</div>
            <div className="list">
              <div className="row">
                <span className="rtext">
                  <span className="rtitle">{t("config.configPath")}</span>
                  <span className="rsub mono">{result.configTomlPath}</span>
                </span>
              </div>
              <div className="row">
                <span className="rtext">
                  <span className="rtitle">{t("config.authPath")}</span>
                  <span className="rsub mono">{result.authJsonPath}</span>
                </span>
              </div>
              {result.backups.map((backup) => (
                <div className="row" key={backup.backupPath}>
                  <span className="rtext">
                    <span className="rtitle">{t("config.backupCreated")}</span>
                    <span className="rsub mono">{backup.backupPath}</span>
                  </span>
                </div>
              ))}
              {result.warning ? (
                <div className="banner info">
                  <Icon name="info" />
                  <span>{result.warning}</span>
                </div>
              ) : null}
            </div>
          </div>
        ) : null}

        <div className="actions">
          <button className="btn big" disabled={busy || !apiKey.trim()} onClick={() => void apply()}>
            {busy ? t("config.applying") : t("config.apply")}
          </button>
        </div>
      </div>
    </div>
  );
}
