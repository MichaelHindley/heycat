import { Plus, Settings } from "lucide-react";
import { Card, CardContent, Button, Input, FormField, MultiSelect } from "../../components/ui";
import { useDictionaryForm } from "../../hooks/useDictionaryForm";
import { EntrySettings } from "./EntrySettings";
import { useDictionaryContext } from "./DictionaryContext";

/**
 * Form for adding new dictionary entries.
 * Uses useDictionaryForm for state and validation.
 */
export function AddEntryForm() {
  const { handleAddEntry, existingTriggers, contextOptions } = useDictionaryContext();

  const form = useDictionaryForm({
    existingTriggers,
    onSubmit: async (values) => {
      await handleAddEntry(
        values.trigger.trim(),
        values.expansion.trim(),
        values.contextIds,
        values.disableSuffix ? undefined : values.suffix.trim() || undefined,
        values.autoEnter || undefined,
        values.disableSuffix || undefined,
        values.completeMatchOnly || undefined
      );
      form.reset();
    },
  });

  return (
    <Card>
      <CardContent className="p-4">
        <form onSubmit={form.handleSubmit}>
          <div className="flex gap-3 items-start">
            <FormField label="Trigger" error={form.triggerError ?? undefined} className="flex-1">
              <Input
                type="text"
                placeholder="e.g., brb"
                value={form.values.trigger}
                onChange={(e) => form.setValue("trigger", e.target.value)}
                aria-label="Trigger phrase"
                aria-invalid={!!form.triggerError}
              />
            </FormField>
            <FormField label="Expansion" className="flex-[2]">
              <Input
                type="text"
                placeholder="e.g., be right back"
                value={form.values.expansion}
                onChange={(e) => form.setValue("expansion", e.target.value)}
                aria-label="Expansion text"
              />
            </FormField>
            <div className="pt-6 flex gap-2">
              <Button
                type="button"
                variant="ghost"
                onClick={form.toggleSettings}
                aria-label="Toggle settings"
                aria-expanded={form.isSettingsOpen}
                className={form.hasSettings ? "text-heycat-orange" : ""}
              >
                <Settings className="h-4 w-4" />
                {form.hasSettings && (
                  <span className="absolute top-1 right-1 w-2 h-2 bg-heycat-orange rounded-full" />
                )}
              </Button>
              <Button type="submit" disabled={form.isSubmitting || !!form.suffixError}>
                <Plus className="h-4 w-4" />
                Add
              </Button>
            </div>
          </div>
          {form.isSettingsOpen && (
            <>
              <EntrySettings
                suffix={form.values.suffix}
                autoEnter={form.values.autoEnter}
                disableSuffix={form.values.disableSuffix}
                completeMatchOnly={form.values.completeMatchOnly}
                onSuffixChange={(value) => form.setValue("suffix", value)}
                onAutoEnterChange={(value) => form.setValue("autoEnter", value)}
                onDisableSuffixChange={(value) => form.setValue("disableSuffix", value)}
                onCompleteMatchOnlyChange={(value) => form.setValue("completeMatchOnly", value)}
                suffixError={form.suffixError}
              />
              {contextOptions.length > 0 && (
                <FormField
                  label="Window Contexts"
                  help="Assign this entry to specific app contexts. Leave empty for global availability."
                  className="mt-3"
                >
                  <MultiSelect
                    selected={form.values.contextIds}
                    onChange={(ids) => form.setValue("contextIds", ids)}
                    options={contextOptions}
                    placeholder="Select contexts (optional)..."
                    aria-label="Window contexts"
                  />
                </FormField>
              )}
            </>
          )}
        </form>
      </CardContent>
    </Card>
  );
}
