import * as React from "react";
import * as CheckBox from "@radix-ui/react-checkbox";
import { Check } from "lucide-react";

interface CaptionCheckBoxProps {
  ischecked: boolean;
  handleChecked: (value: boolean) => void;
}

const CaptionCheckBox: React.FC<CaptionCheckBoxProps> = ({
  ischecked,
  handleChecked,
}) => {
  const onCheckedChange = (value: boolean) => {
    handleChecked(value);
  };
  return (
    <div className="flex items-center">
      <CheckBox.Root
        className="rounded-md bg-gray-50 shadow-white w-6 h-6 align-middle mr-2"
        checked={ischecked}
        id="use-caption"
        onCheckedChange={onCheckedChange}
      >
        <CheckBox.Indicator>
          {ischecked && <Check className="text-gray-500 w-full h-full p-1" />}
        </CheckBox.Indicator>
      </CheckBox.Root>
      <label htmlFor="use-caption" className="text-gray-100">
        Auto
      </label>
    </div>
  );
};

export default CaptionCheckBox;
