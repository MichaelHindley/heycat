import { useCallback, useState } from "react";
import { validateTrigger, validateSuffix, isDuplicateTrigger } from "../lib/validation";

/**
 * Form values for dictionary entry creation/editing.
 */
export interface DictionaryFormValues {
  trigger: string;
  expansion: string;
  suffix: string;
  autoEnter: boolean;
  disableSuffix: boolean;
  completeMatchOnly: boolean;
  contextIds: string[];
}

/**
 * Configuration options for useDictionaryForm hook.
 */
export interface UseDictionaryFormOptions {
  /** Existing triggers for duplicate detection (lowercase) */
  existingTriggers: string[];
  /** ID to exclude from duplicate check (for editing) */
  excludeId?: string;
  /** Initial values for editing mode */
  initialValues?: Partial<DictionaryFormValues>;
  /** Callback when form is submitted successfully */
  onSubmit?: (values: DictionaryFormValues) => Promise<void>;
}

/**
 * Return type of the useDictionaryForm hook.
 */
export interface UseDictionaryFormReturn {
  /** Current form values */
  values: DictionaryFormValues;
  /** Trigger validation error */
  triggerError: string | null;
  /** Suffix validation error */
  suffixError: string | null;
  /** Whether the form is submitting */
  isSubmitting: boolean;
  /** Whether settings panel is open */
  isSettingsOpen: boolean;
  /** Set a form value */
  setValue: <K extends keyof DictionaryFormValues>(
    field: K,
    value: DictionaryFormValues[K]
  ) => void;
  /** Toggle settings panel */
  toggleSettings: () => void;
  /** Handle form submission */
  handleSubmit: (e?: React.FormEvent) => Promise<boolean>;
  /** Reset form to initial state */
  reset: () => void;
  /** Check if form has non-default settings */
  hasSettings: boolean;
}

const DEFAULT_VALUES: DictionaryFormValues = {
  trigger: "",
  expansion: "",
  suffix: "",
  autoEnter: false,
  disableSuffix: false,
  completeMatchOnly: false,
  contextIds: [],
};

/**
 * Hook for managing dictionary entry form state with validation.
 *
 * Integrates with the validation utilities from lib/validation for
 * consistent trigger and suffix validation.
 *
 * @example
 * const form = useDictionaryForm({
 *   existingTriggers: entries.map(e => e.trigger.toLowerCase()),
 *   onSubmit: async (values) => {
 *     await addEntry.mutateAsync(values);
 *   },
 * });
 */
export function useDictionaryForm(
  options: UseDictionaryFormOptions
): UseDictionaryFormReturn {
  const { existingTriggers, excludeId, initialValues, onSubmit } = options;

  const [values, setValues] = useState<DictionaryFormValues>({
    ...DEFAULT_VALUES,
    ...initialValues,
  });
  const [triggerError, setTriggerError] = useState<string | null>(null);
  const [suffixError, setSuffixError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  const setValue = useCallback(
    <K extends keyof DictionaryFormValues>(field: K, value: DictionaryFormValues[K]) => {
      setValues((prev) => ({ ...prev, [field]: value }));

      // Clear errors when fields are modified
      if (field === "trigger") {
        setTriggerError(null);
      }
      if (field === "suffix" && typeof value === "string") {
        const error = validateSuffix(value);
        setSuffixError(error);
      }
    },
    []
  );

  const toggleSettings = useCallback(() => {
    setIsSettingsOpen((prev) => !prev);
  }, []);

  const validate = useCallback((): boolean => {
    // Validate trigger
    const triggerValidation = validateTrigger(values.trigger);
    if (triggerValidation) {
      setTriggerError(triggerValidation);
      return false;
    }

    // Check for duplicates (excluding current entry if editing)
    const triggersToCheck = excludeId
      ? existingTriggers.filter((t) => t !== values.trigger.toLowerCase())
      : existingTriggers;

    if (isDuplicateTrigger(values.trigger, triggersToCheck)) {
      setTriggerError("This trigger already exists");
      return false;
    }

    // Validate suffix
    const suffixValidation = validateSuffix(values.suffix);
    if (suffixValidation) {
      setSuffixError(suffixValidation);
      return false;
    }

    return true;
  }, [values, existingTriggers, excludeId]);

  const handleSubmit = useCallback(
    async (e?: React.FormEvent): Promise<boolean> => {
      if (e) {
        e.preventDefault();
      }

      if (!validate()) {
        return false;
      }

      if (!onSubmit) {
        return true;
      }

      setIsSubmitting(true);
      try {
        await onSubmit(values);
        return true;
      } finally {
        setIsSubmitting(false);
      }
    },
    [validate, onSubmit, values]
  );

  const reset = useCallback(() => {
    setValues({ ...DEFAULT_VALUES, ...initialValues });
    setTriggerError(null);
    setSuffixError(null);
    setIsSettingsOpen(false);
  }, [initialValues]);

  const hasSettings =
    values.suffix !== "" ||
    values.autoEnter ||
    values.disableSuffix ||
    values.completeMatchOnly;

  return {
    values,
    triggerError,
    suffixError,
    isSubmitting,
    isSettingsOpen,
    setValue,
    toggleSettings,
    handleSubmit,
    reset,
    hasSettings,
  };
}
