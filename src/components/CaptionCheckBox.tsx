import * as CheckBox from "@radix-ui/react-checkbox";
import { Check } from "lucide-react";

const CaptionCheckBox = ({
  handleChecked,
}: {
  handleChecked: (checked: boolean) => void;
}) => {
  return (
    <div className="flex items-center">
      <CheckBox.Root
        className="rounded-md bg-gray-50 shadow-white w-8 h-8 align-middle mr-2"
        defaultChecked
        id="use-caption"
        onCheckedChange={handleChecked}
      >
        <CheckBox.Indicator>
          <Check className="text-gray-500 w-full h-full p-1" />
        </CheckBox.Indicator>
      </CheckBox.Root>
      <label htmlFor="use-caption" className="text-gray-100">
        Use subtitle
      </label>
    </div>
  );
};

export default CaptionCheckBox;
