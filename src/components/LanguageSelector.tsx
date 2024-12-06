import * as React from "react";
import * as Select from "@radix-ui/react-select";
import { ChevronDownIcon, CheckIcon, Languages } from "lucide-react";

interface Language {
  code: string;
  name: string;
}

const LANGUAGES: Language[] = [
  { code: "en", name: "English" },
  { code: "es", name: "Español" },
  { code: "fr", name: "Français" },
  { code: "de", name: "Deutsch" },
  { code: "zh", name: "中文" },
  { code: "ar", name: "العربية" },
  { code: "ru", name: "Русский" },
  { code: "ja", name: "日本語" },
];

interface LanguageSelectorProps {
  selectedLanguage: string;
  onLanguageChange: (language: string) => void;
}

const LanguageSelector: React.FC<LanguageSelectorProps> = ({
  selectedLanguage,
  onLanguageChange,
}) => {
  return (
    <div className="flex items-center gap-5">
      <Languages className="w-7 h-7 text-green-200" />
      <Select.Root value={selectedLanguage} onValueChange={onLanguageChange}>
        <Select.Trigger
          className="flex items-center w-24 justify-between px-3 py-2 
                     text-left bg-white border border-gray-300 rounded-md 
                     shadow-sm focus:outline-none"
          aria-label="Select language"
        >
          <Select.Value>
            {LANGUAGES.find((lang) => lang.code === selectedLanguage)?.name}
          </Select.Value>
          <Select.Icon>
            <ChevronDownIcon className="w-4 h-4 text-gray-400" />
          </Select.Icon>
        </Select.Trigger>

        <Select.Portal>
          <Select.Content
            className="bg-white rounded-md shadow-lg 
                       z-50 overflow-hidden"
            position="popper"
          >
            <Select.Viewport className="p-1 focus:outline-none">
              {LANGUAGES.map((language) => (
                <Select.Item
                  key={language.code}
                  value={language.code}
                  className="relative flex items-center px-3 py-2 
                             select-none hover:bg-gray-100 
                             focus:bg-gray-100 cursor-pointer 
                             text-gray-900 rounded-md 
                             data-[highlighted]:outline-none 
                             data-[highlighted]:bg-gray-100"
                >
                  <Select.ItemText>{language.name}</Select.ItemText>
                  <Select.ItemIndicator className="absolute right-2">
                    <CheckIcon className="w-4 h-4 text-blue-600" />
                  </Select.ItemIndicator>
                </Select.Item>
              ))}
            </Select.Viewport>
          </Select.Content>
        </Select.Portal>
      </Select.Root>
    </div>
  );
};

export default LanguageSelector;
