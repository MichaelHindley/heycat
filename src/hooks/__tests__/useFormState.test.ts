/**
 * Tests for useFormState hook.
 */

import { describe, it, expect, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useFormState } from "../useFormState";

describe("useFormState", () => {
  const initialValues = { name: "", email: "" };

  it("initializes with provided values", () => {
    const { result } = renderHook(() =>
      useFormState({ initialValues })
    );

    expect(result.current.values).toEqual(initialValues);
    expect(result.current.errors).toEqual({});
    expect(result.current.isSubmitting).toBe(false);
    expect(result.current.isDirty).toBe(false);
  });

  it("updates values and tracks dirty state", () => {
    const { result } = renderHook(() =>
      useFormState({ initialValues })
    );

    act(() => {
      result.current.setValue("name", "John");
    });

    expect(result.current.values.name).toBe("John");
    expect(result.current.isDirty).toBe(true);
  });

  it("validates on submit and prevents submission if invalid", async () => {
    const onSubmit = vi.fn();
    const { result } = renderHook(() =>
      useFormState({
        initialValues,
        validate: (values) => ({
          name: values.name ? undefined : "Name is required",
        }),
        onSubmit,
      })
    );

    let success: boolean;
    await act(async () => {
      success = await result.current.handleSubmit();
    });

    expect(success!).toBe(false);
    expect(result.current.errors.name).toBe("Name is required");
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("calls onSubmit when validation passes", async () => {
    const onSubmit = vi.fn();
    const { result } = renderHook(() =>
      useFormState({
        initialValues: { name: "John", email: "john@test.com" },
        validate: () => ({}),
        onSubmit,
      })
    );

    let success: boolean;
    await act(async () => {
      success = await result.current.handleSubmit();
    });

    expect(success!).toBe(true);
    expect(onSubmit).toHaveBeenCalledWith({
      name: "John",
      email: "john@test.com",
    });
  });

  it("clears errors when field is modified", () => {
    const { result } = renderHook(() =>
      useFormState({
        initialValues,
        validate: (values) => ({
          name: values.name ? undefined : "Required",
        }),
      })
    );

    // Trigger validation to set error
    act(() => {
      result.current.validateAll();
    });
    expect(result.current.errors.name).toBe("Required");

    // Modify field to clear error
    act(() => {
      result.current.setValue("name", "test");
    });
    expect(result.current.errors.name).toBeUndefined();
  });

  it("resets form to initial values", () => {
    const { result } = renderHook(() =>
      useFormState({ initialValues })
    );

    act(() => {
      result.current.setValue("name", "John");
      result.current.setError("name", "Some error");
    });

    expect(result.current.values.name).toBe("John");
    expect(result.current.errors.name).toBe("Some error");

    act(() => {
      result.current.reset();
    });

    expect(result.current.values).toEqual(initialValues);
    expect(result.current.errors).toEqual({});
  });
});
