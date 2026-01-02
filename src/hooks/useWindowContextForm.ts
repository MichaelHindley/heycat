import { useCallback, useState } from "react";
import { validateRegexPattern } from "../lib/validation";
import type { OverrideMode } from "../types/windowContext";

/**
 * Form values for window context creation/editing.
 */
export interface WindowContextFormValues {
  name: string;
  appName: string;
  bundleId?: string;
  titlePattern: string;
  commandMode: OverrideMode;
  dictionaryMode: OverrideMode;
  dictionaryEntryIds: string[];
  priority: number;
  enabled: boolean;
}

/**
 * Configuration options for useWindowContextForm hook.
 */
export interface UseWindowContextFormOptions {
  /** Initial values for editing mode */
  initialValues?: Partial<WindowContextFormValues>;
  /** Callback when form is submitted successfully */
  onSubmit?: (values: WindowContextFormValues) => Promise<void>;
}

/**
 * Return type of the useWindowContextForm hook.
 */
export interface UseWindowContextFormReturn {
  /** Current form values */
  values: WindowContextFormValues;
  /** Name validation error */
  nameError: string | null;
  /** App name validation error */
  appNameError: string | null;
  /** Pattern validation error */
  patternError: string | null;
  /** Whether the form is submitting */
  isSubmitting: boolean;
  /** Set a form value */
  setValue: <K extends keyof WindowContextFormValues>(
    field: K,
    value: WindowContextFormValues[K]
  ) => void;
  /** Handle app selection from combobox */
  handleAppSelect: (appName: string, bundleId?: string) => void;
  /** Handle form submission */
  handleSubmit: (e?: React.FormEvent) => Promise<boolean>;
  /** Reset form to initial state */
  reset: () => void;
}

const DEFAULT_VALUES: WindowContextFormValues = {
  name: "",
  appName: "",
  bundleId: undefined,
  titlePattern: "",
  commandMode: "merge",
  dictionaryMode: "merge",
  dictionaryEntryIds: [],
  priority: 0,
  enabled: true,
};

/**
 * Hook for managing window context form state with validation.
 *
 * Integrates with the validation utilities from lib/validation for
 * consistent regex pattern validation.
 *
 * @example
 * const form = useWindowContextForm({
 *   onSubmit: async (values) => {
 *     await createContext.mutateAsync(values);
 *   },
 * });
 */
export function useWindowContextForm(
  options: UseWindowContextFormOptions = {}
): UseWindowContextFormReturn {
  const { initialValues, onSubmit } = options;

  const [values, setValues] = useState<WindowContextFormValues>({
    ...DEFAULT_VALUES,
    ...initialValues,
  });
  const [nameError, setNameError] = useState<string | null>(null);
  const [appNameError, setAppNameError] = useState<string | null>(null);
  const [patternError, setPatternError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const setValue = useCallback(
    <K extends keyof WindowContextFormValues>(
      field: K,
      value: WindowContextFormValues[K]
    ) => {
      setValues((prev) => {
        // Clear bundleId when appName is manually typed
        if (field === "appName") {
          return { ...prev, appName: value as string, bundleId: undefined };
        }
        return { ...prev, [field]: value };
      });

      // Clear errors when fields are modified
      if (field === "name") {
        setNameError(null);
      }
      if (field === "appName") {
        setAppNameError(null);
      }
      if (field === "titlePattern" && typeof value === "string") {
        const error = validateRegexPattern(value);
        setPatternError(error);
      }
    },
    []
  );

  const handleAppSelect = useCallback((appName: string, bundleId?: string) => {
    setValues((prev) => ({ ...prev, appName, bundleId }));
    setAppNameError(null);
  }, []);

  const validate = useCallback((): boolean => {
    // Validate name
    if (!values.name.trim()) {
      setNameError("Name is required");
      return false;
    }

    // Validate app name
    if (!values.appName.trim()) {
      setAppNameError("App name is required");
      return false;
    }

    // Validate pattern
    const patternValidation = validateRegexPattern(values.titlePattern);
    if (patternValidation) {
      setPatternError(patternValidation);
      return false;
    }

    return true;
  }, [values]);

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
    setNameError(null);
    setAppNameError(null);
    setPatternError(null);
  }, [initialValues]);

  return {
    values,
    nameError,
    appNameError,
    patternError,
    isSubmitting,
    setValue,
    handleAppSelect,
    handleSubmit,
    reset,
  };
}
