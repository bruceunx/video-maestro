import * as React from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { Settings, X } from "lucide-react";

const SettingsModal = () => {
  const [isOpen, setIsOpen] = React.useState(false);

  const [darkMode, setDarkMode] = React.useState(false);
  const [notifications, setNotifications] = React.useState(true);

  const handleSave = () => {
    console.log("Settings saved:", { darkMode, notifications });
    setIsOpen(false);
  };

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
            <Dialog.Title className="text-xl font-semibold">
              Application Settings
            </Dialog.Title>
            <Dialog.Close
              className="text-gray-500 hover:text-gray-700 
                         transition-colors rounded-full p-1"
            >
              <X className="w-5 h-5" />
            </Dialog.Close>
          </div>

          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <label htmlFor="dark-mode" className="text-gray-700">
                Dark Mode
              </label>
              <button
                id="dark-mode"
                onClick={() => setDarkMode(!darkMode)}
                className={`w-12 h-6 rounded-full transition-colors ${
                  darkMode ? "bg-blue-500" : "bg-gray-300"
                }`}
              >
                <span
                  className={`block w-5 h-5 bg-white rounded-full shadow-md transform transition-transform ${
                    darkMode ? "translate-x-6" : "translate-x-1"
                  }`}
                />
              </button>
            </div>

            <div className="flex justify-between items-center">
              <label htmlFor="notifications" className="text-gray-700">
                Enable Notifications
              </label>
              <button
                id="notifications"
                onClick={() => setNotifications(!notifications)}
                className={`w-12 h-6 rounded-full transition-colors ${
                  notifications ? "bg-blue-500" : "bg-gray-300"
                }`}
              >
                <span
                  className={`block w-5 h-5 bg-white rounded-full shadow-md transform transition-transform ${
                    notifications ? "translate-x-6" : "translate-x-1"
                  }`}
                />
              </button>
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
