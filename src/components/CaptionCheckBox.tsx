import * as CheckBox from "@radix-ui/react-checkbox";
import { CheckIcon } from "@radix-ui/react-icons";

const CaptionCheckBox = () => {
  return (
    <div>
      <CheckBox.Root>
        <CheckBox.Indicator>
          <CheckIcon />
        </CheckBox.Indicator>
      </CheckBox.Root>
    </div>
  );
};

export default CaptionCheckBox;
