import * as SelectPrimitive from "@radix-ui/react-select";
import { forwardRef } from "react";
import { ChevronDown, Check } from "lucide-react";

export interface SelectProps {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  children: React.ReactNode;
}

export const Select = ({
  children,
  placeholder,
  ...props
}: SelectProps) => (
  <SelectPrimitive.Root {...props}>
    <SelectTrigger placeholder={placeholder} />
    <SelectContent>{children}</SelectContent>
  </SelectPrimitive.Root>
);

Select.displayName = "Select";

interface SelectTriggerProps {
  placeholder?: string;
}

const SelectTrigger = forwardRef<HTMLButtonElement, SelectTriggerProps>(
  ({ placeholder }, ref) => (
    <SelectPrimitive.Trigger
      ref={ref}
      className="
        inline-flex items-center justify-between gap-2
        w-full
        bg-surface
        border border-border rounded-[var(--radius-sm)]
        px-3.5 py-2.5
        text-base text-text-primary
        transition-all duration-[var(--duration-fast)] ease-[var(--ease-default)]
        focus:outline-none focus:border-heycat-teal focus:ring-2 focus:ring-heycat-teal/10
        disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-text-secondary/10
        data-[placeholder]:text-text-secondary
      "
    >
      <SelectPrimitive.Value placeholder={placeholder} />
      <SelectPrimitive.Icon>
        <ChevronDown className="h-4 w-4 text-text-secondary" />
      </SelectPrimitive.Icon>
    </SelectPrimitive.Trigger>
  )
);

SelectTrigger.displayName = "SelectTrigger";

const SelectContent = forwardRef<
  HTMLDivElement,
  SelectPrimitive.SelectContentProps
>(({ children, ...props }, ref) => (
  <SelectPrimitive.Portal>
    <SelectPrimitive.Content
      ref={ref}
      className="
        overflow-hidden
        bg-surface
        border border-border rounded-[var(--radius-md)]
        shadow-lg
        animate-in fade-in-0 zoom-in-95
        data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95
      "
      position="popper"
      sideOffset={4}
      {...props}
    >
      <SelectPrimitive.Viewport className="p-1">
        {children}
      </SelectPrimitive.Viewport>
    </SelectPrimitive.Content>
  </SelectPrimitive.Portal>
));

SelectContent.displayName = "SelectContent";

export interface SelectItemProps {
  value: string;
  children: React.ReactNode;
  disabled?: boolean;
}

export const SelectItem = forwardRef<HTMLDivElement, SelectItemProps>(
  ({ children, ...props }, ref) => (
    <SelectPrimitive.Item
      ref={ref}
      className="
        relative flex items-center
        px-8 py-2
        text-sm text-text-primary
        rounded-[var(--radius-sm)]
        cursor-pointer
        select-none
        outline-none
        transition-colors duration-[var(--duration-fast)]
        focus:bg-heycat-orange/10
        data-[highlighted]:bg-heycat-orange/10
        data-[disabled]:opacity-50 data-[disabled]:cursor-not-allowed
      "
      {...props}
    >
      <SelectPrimitive.ItemIndicator className="absolute left-2 inline-flex items-center justify-center">
        <Check className="h-4 w-4 text-heycat-orange" />
      </SelectPrimitive.ItemIndicator>
      <SelectPrimitive.ItemText>{children}</SelectPrimitive.ItemText>
    </SelectPrimitive.Item>
  )
);

SelectItem.displayName = "SelectItem";

export interface SelectGroupProps {
  label: string;
  children: React.ReactNode;
}

export const SelectGroup = ({ label, children }: SelectGroupProps) => (
  <SelectPrimitive.Group>
    <SelectPrimitive.Label className="px-8 py-1.5 text-xs font-medium text-text-secondary uppercase tracking-wider">
      {label}
    </SelectPrimitive.Label>
    {children}
  </SelectPrimitive.Group>
);

SelectGroup.displayName = "SelectGroup";

export const SelectSeparator = () => (
  <SelectPrimitive.Separator className="h-px my-1 bg-border" />
);

SelectSeparator.displayName = "SelectSeparator";
