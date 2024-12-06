import * as React from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { Settings, X, Eye, EyeOff } from "lucide-react";
import { useSettings } from "store/SettingsProvider";
import { SettingsType } from "types/settings";

const SettingsModal: React.FC = () => {
  const [isOpen, setIsOpen] = React.useState(false);
  const [showApiKey, setShowApiKey] = React.useState(false);

  const { settings: saveSettings, updateSettings } = useSettings();
  const [settings, setSettings] = React.useState<SettingsType>(saveSettings);

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setSettings({ ...settings, [name]: value });
  };

  const handleSave = () => {
    updateSettings(settings);
    setIsOpen(false);
  };

  // const toggleSetting = (key: keyof Settings) => {
  //   setSettings((prev) => ({
  //     ...prev,
  //     [key]: !prev[key],
  //   }));
  // };
  //
  React.useEffect(() => {
    setSettings({ ...settings, ...saveSettings });
  }, [saveSettings]);

  return (
    <Dialog.Root open={isOpen} onOpenChange={setIsOpen}>
      <Dialog.Trigger asChild>
        <button
          className="p-2 rounded-full transition-colors"
          onClick={() => setIsOpen(true)}
        >
          <Settings className="w-6 h-6 text-gray-500 hover:text-gray-400 active:text-gray-300" />
        </button>
      </Dialog.Trigger>

      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 animate-overlay-show" />

        <Dialog.Content
          className="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 
                     w-full max-w-md bg-white rounded-lg shadow-xl 
                     p-6 z-50 focus:outline-none animate-content-show"
        >
          <div className="flex justify-between items-center mb-4">
            <Dialog.Title className="text-xl font-semibold focus:no-underline">
              Application Settings
            </Dialog.Title>
            <Dialog.Close
              className="text-gray-500 hover:text-gray-700 
                         transition-colors rounded-full p-1"
            >
              <X className="w-5 h-5 focus:outline-none" />
            </Dialog.Close>
          </div>

          <div className="space-y-4">
            <div className="space-y-3">
              <div className="relative">
                <label
                  htmlFor="apiKey"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  API Key
                </label>
                <div className="flex items-center">
                  <input
                    type={showApiKey ? "text" : "password"}
                    id="apiKey"
                    name="apiKey"
                    value={settings.apiKey}
                    onChange={handleInputChange}
                    className="flex-grow px-3 py-2 border border-gray-300 rounded-md shadow-sm 
                               focus:outline-none focus:ring-2 focus:ring-blue-500"
                    placeholder="Enter your API key"
                  />
                  <button
                    type="button"
                    onClick={() => setShowApiKey(!showApiKey)}
                    className="ml-2 text-gray-500 hover:text-gray-700"
                  >
                    {showApiKey ? (
                      <EyeOff className="w-5 h-5" />
                    ) : (
                      <Eye className="w-5 h-5" />
                    )}
                  </button>
                </div>
              </div>

              <div>
                <label
                  htmlFor="aiSupplierUrl"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  AI Supplier URL
                </label>
                <input
                  type="text"
                  id="aiSupplierUrl"
                  name="aiSupplierUrl"
                  value={settings.aiSupplierUrl}
                  onChange={handleInputChange}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm 
                             focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="Enter AI supplier URL"
                />
              </div>

              <div>
                <label
                  htmlFor="aiModelName"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  AI Model Name
                </label>
                <input
                  type="text"
                  id="aiModelName"
                  name="aiModelName"
                  value={settings.aiModelName}
                  onChange={handleInputChange}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm 
                             focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="Enter model name like gpt-4o"
                />
              </div>

              <div>
                <label
                  htmlFor="whisperModelName"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  Whisper Model Name
                </label>
                <input
                  type="text"
                  id="whisperModelName"
                  name="whisperModelName"
                  value={settings.whisperModelName}
                  onChange={handleInputChange}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm 
                             focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="Enter whisper model name"
                />
              </div>
            </div>
          </div>

          <div className="flex justify-end space-x-2 mt-6">
            <Dialog.Close
              className="px-4 py-2 bg-gray-200 text-gray-700 
                         rounded hover:bg-gray-300 transition-colors"
            >
              Cancel
            </Dialog.Close>
            <button
              onClick={handleSave}
              className="px-4 py-2 bg-blue-500 text-white 
                         rounded hover:bg-blue-600 transition-colors"
            >
              Save Settings
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export default SettingsModal;
