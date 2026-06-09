import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const DEFAULT_API_URL = "https://api.deepseek.com/v1/chat/completions";

interface AppConfig {
  api_key: string;
  api_base_url: string;
  model: string;
  enabled: boolean;
}

function App() {
  const [config, setConfig] = useState<AppConfig>({
    api_key: "",
    api_base_url: DEFAULT_API_URL,
    model: "deepseek-v4-flash",
    enabled: true,
  });
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");
  const [showKey, setShowKey] = useState(false);

  useEffect(() => {
    invoke<AppConfig>("get_config").then(setConfig).catch(console.error);
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setMessage("");
    try {
      await invoke("save_config", { cfg: config });
      setMessage("Settings saved");
      setTimeout(() => setMessage(""), 2000);
    } catch (e) {
      setMessage(`Error: ${e}`);
    }
    setSaving(false);
  };

  const handleToggle = async () => {
    try {
      const newState = await invoke<boolean>("toggle_enabled");
      setConfig((c) => ({ ...c, enabled: newState }));
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      <div className="max-w-md mx-auto">
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-xl font-semibold text-gray-900 dark:text-white">
            cn2en Settings
          </h1>
          <button
            onClick={handleToggle}
            className={`px-3 py-1.5 rounded-full text-sm font-medium transition-colors ${
              config.enabled
                ? "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300"
                : "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300"
            }`}
          >
            {config.enabled ? "Active" : "Paused"}
          </button>
        </div>

        <div className="space-y-5">
          {/* API Key */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
              API Key
            </label>
            <div className="relative">
              <input
                type={showKey ? "text" : "password"}
                value={config.api_key}
                onChange={(e) =>
                  setConfig((c) => ({ ...c, api_key: e.target.value }))
                }
                placeholder="sk-..."
                className="w-full px-3 py-2 pr-16 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none"
              />
              <button
                type="button"
                onClick={() => setShowKey(!showKey)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 px-2 py-1"
              >
                {showKey ? "Hide" : "Show"}
              </button>
            </div>
            <p className="mt-1.5 text-xs text-gray-500 dark:text-gray-400">
              密钥不会打进安装包里，只会保存在本机的配置文件中；分发给别人时请让对方自行填写。
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
              API 地址（OpenAI 兼容 chat/completions）
            </label>
            <input
              type="url"
              value={config.api_base_url}
              onChange={(e) =>
                setConfig((c) => ({ ...c, api_base_url: e.target.value }))
              }
              placeholder={DEFAULT_API_URL}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none"
            />
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              默认 DeepSeek；也可填自建代理或其它兼容服务的完整 URL。
            </p>
          </div>

          {/* Model */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
              Model
            </label>
            <select
              value={config.model}
              onChange={(e) =>
                setConfig((c) => ({ ...c, model: e.target.value }))
              }
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none"
            >
              <option value="deepseek-v4-flash">DeepSeek V4 Flash (Fast)</option>
              <option value="deepseek-v4-pro">DeepSeek V4 Pro (Quality)</option>
            </select>
          </div>

          <div className="rounded-lg border border-gray-200 dark:border-gray-700 bg-white/60 dark:bg-gray-800/60 p-3 text-sm text-gray-600 dark:text-gray-300">
            <div className="font-medium text-gray-800 dark:text-gray-100 mb-1">
              Shortcut
            </div>
            <div>
              Select Chinese text anywhere, then press{" "}
              <span className="font-semibold">Ctrl + Alt + T</span>.
              cn2en will copy the selection, translate it, paste the English
              result over the selection, and restore your clipboard text.
            </div>
          </div>

          {/* Save */}
          <button
            onClick={handleSave}
            disabled={saving}
            className="w-full py-2.5 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white rounded-lg text-sm font-medium transition-colors"
          >
            {saving ? "Saving..." : "Save"}
          </button>

          {message && (
            <p
              className={`text-sm text-center ${
                message.startsWith("Error")
                  ? "text-red-500"
                  : "text-green-600 dark:text-green-400"
              }`}
            >
              {message}
            </p>
          )}
        </div>

        <p className="mt-6 text-xs text-gray-400 text-center">
          cn2en stays in the tray and translates selected Chinese text when you
          press Ctrl + Alt + T.
        </p>
      </div>
    </div>
  );
}

export default App;
