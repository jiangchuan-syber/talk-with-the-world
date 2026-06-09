import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

const DEEPSEEK_API_PLATFORM_URL = "https://platform.deepseek.com/api_keys";
const TOGGLE_HINT_KEY = "cn2en.dismissed_toggle_hint";

interface AppConfig {
  api_key: string;
  model: string;
  enabled: boolean;
}

function App() {
  const [config, setConfig] = useState<AppConfig>({
    api_key: "",
    model: "deepseek-v4-flash",
    enabled: true,
  });
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [showToggleHint, setShowToggleHint] = useState(false);

  useEffect(() => {
    invoke<AppConfig>("get_config").then(setConfig).catch(console.error);
    if (!localStorage.getItem(TOGGLE_HINT_KEY)) {
      setShowToggleHint(true);
    }
  }, []);

  const dismissToggleHint = () => {
    localStorage.setItem(TOGGLE_HINT_KEY, "1");
    setShowToggleHint(false);
  };

  const handleSave = async () => {
    setSaving(true);
    setMessage("");
    try {
      await invoke("save_config", { cfg: config });
      setMessage("设置已保存");
      setTimeout(() => setMessage(""), 2000);
    } catch (e) {
      setMessage(`保存失败：${e}`);
    }
    setSaving(false);
  };

  const handleToggle = async () => {
    dismissToggleHint();
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
            划译设置
          </h1>
          <button
            onClick={handleToggle}
            className={`px-3 py-1.5 rounded-full text-sm font-medium transition-colors ${
              config.enabled
                ? "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300"
                : "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300"
            }`}
          >
            {config.enabled ? "已启用" : "已暂停"}
          </button>
        </div>

        {showToggleHint && (
          <div className="mb-5 rounded-lg border border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-950/50 p-3 text-sm text-green-800 dark:text-green-200">
            <p>
              点击右上角绿色「已启用」按钮，可切换划译的开启与暂停；暂停后快捷键不会触发翻译。
            </p>
            <button
              type="button"
              onClick={dismissToggleHint}
              className="mt-2 text-xs font-medium text-green-700 dark:text-green-300 hover:underline"
            >
              知道了
            </button>
          </div>
        )}

        <div className="space-y-5">
          <div>
            <div className="flex items-center justify-between mb-1.5">
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                API 密钥
              </label>
              <button
                type="button"
                onClick={() => openUrl(DEEPSEEK_API_PLATFORM_URL)}
                className="text-xs text-blue-600 dark:text-blue-400 hover:underline"
              >
                前往 DeepSeek 开通 API
              </button>
            </div>
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
                {showKey ? "隐藏" : "显示"}
              </button>
            </div>
            <p className="mt-1.5 text-xs text-gray-500 dark:text-gray-400">
              密钥不会打进安装包里，只会保存在本机的配置文件中；分发给别人时请让对方自行填写。
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
              翻译模型
            </label>
            <select
              value={config.model}
              onChange={(e) =>
                setConfig((c) => ({ ...c, model: e.target.value }))
              }
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none"
            >
              <option value="deepseek-v4-flash">DeepSeek V4 Flash（快速）</option>
              <option value="deepseek-v4-pro">DeepSeek V4 Pro（高质量）</option>
            </select>
          </div>

          <div className="rounded-lg border border-gray-200 dark:border-gray-700 bg-white/60 dark:bg-gray-800/60 p-3 text-sm text-gray-600 dark:text-gray-300">
            <div className="font-medium text-gray-800 dark:text-gray-100 mb-1">
              快捷键
            </div>
            <div>
              在任意位置选中中文后，按下{" "}
              <span className="font-semibold">Ctrl + Shift + A</span>
              。划译会复制选区、翻译成英文、粘贴替换原文，并恢复你原来的剪贴板内容。
            </div>
          </div>

          <button
            onClick={handleSave}
            disabled={saving}
            className="w-full py-2.5 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white rounded-lg text-sm font-medium transition-colors"
          >
            {saving ? "保存中…" : "保存设置"}
          </button>

          {message && (
            <p
              className={`text-sm text-center ${
                message.startsWith("保存失败")
                  ? "text-red-500"
                  : "text-green-600 dark:text-green-400"
              }`}
            >
              {message}
            </p>
          )}
        </div>

        <p className="mt-6 text-xs text-gray-400 text-center">
          划译常驻系统托盘，选中中文后按 Ctrl + Shift + A 即可翻译。
        </p>
      </div>
    </div>
  );
}

export default App;
