---
status: completed
created: 2025-12-22
completed: 2025-12-22
dependencies: ["backend-storage-update"]
review_round: 1
---

# Spec: Update DictionaryExpander to append suffix when expanding

## Description

Modify DictionaryExpander to:
1. Return an `ExpansionResult` struct instead of a plain `String`
2. Append the entry's suffix to the expansion text when present
3. Track whether any expanded entry has `auto_enter: true` for the keyboard simulation spec

## Acceptance Criteria

- [ ] `expand()` returns `ExpansionResult` struct with `expanded_text` and `should_press_enter` fields
- [ ] When entry has suffix, expansion includes suffix (e.g., "brb" with suffix "." → "be right back.")
- [ ] When entry has no suffix, expansion is unchanged
- [ ] When any expanded entry has `auto_enter: true`, result has `should_press_enter: true`
- [ ] Multiple expansions in same text all apply their suffixes correctly

## Test Cases

- [ ] Expand "brb" with suffix "." → "be right back."
- [ ] Expand "brb" without suffix → "be right back"
- [ ] Expand "brb" with auto_enter=true → should_press_enter is true
- [ ] Expand text with multiple triggers, one has auto_enter → should_press_enter is true
- [ ] Expand text with no matches → should_press_enter is false

## Dependencies

- `backend-storage-update` - DictionaryStore must provide entries with suffix/auto_enter fields

## Preconditions

- DictionaryEntry struct has suffix and auto_enter fields
- DictionaryStore provides entries with these fields populated

## Implementation Notes

### Data Flow Position
```
DictionaryStore (provides entries)
       ↓
DictionaryExpander.expand() ← This spec
       ↓
ExpansionResult { expanded_text, should_press_enter }
       ↓
TranscriptionService (uses result)
```

### New ExpansionResult Struct (`src-tauri/src/dictionary/expander.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpansionResult {
    pub expanded_text: String,
    pub should_press_enter: bool,
}
```

### Updated expand() Method

```rust
impl DictionaryExpander {
    pub fn expand(&self, text: &str) -> ExpansionResult {
        let mut result = text.to_string();
        let mut should_press_enter = false;

        for pattern in &self.patterns {
            if pattern.regex.is_match(&result) {
                // Build replacement with suffix
                let replacement = match &pattern.entry.suffix {
                    Some(suffix) => format!("{}{}", pattern.entry.expansion, suffix),
                    None => pattern.entry.expansion.clone(),
                };

                result = pattern.regex.replace_all(&result, replacement.as_str()).to_string();

                // Track auto_enter
                if pattern.entry.auto_enter {
                    should_press_enter = true;
                }
            }
        }

        ExpansionResult {
            expanded_text: result,
            should_press_enter,
        }
    }
}
```

### Pattern Struct Update

The internal `CompiledPattern` struct needs to store the full entry:

```rust
struct CompiledPattern {
    regex: Regex,
    entry: DictionaryEntry,  // Changed from just storing expansion string
}
```

### Testing Strategy

**Backend (Rust):**
```rust
// src-tauri/src/dictionary/expander_test.rs
#[test]
fn test_expand_with_suffix() {
    let entries = vec![DictionaryEntry {
        id: "1".to_string(),
        trigger: "brb".to_string(),
        expansion: "be right back".to_string(),
        suffix: Some(".".to_string()),
        auto_enter: false,
    }];

    let expander = DictionaryExpander::new(entries);
    let result = expander.expand("I'll brb");

    assert_eq!(result.expanded_text, "I'll be right back.");
    assert_eq!(result.should_press_enter, false);
}

#[test]
fn test_expand_without_suffix() {
    let entries = vec![DictionaryEntry {
        id: "1".to_string(),
        trigger: "brb".to_string(),
        expansion: "be right back".to_string(),
        suffix: None,
        auto_enter: false,
    }];

    let expander = DictionaryExpander::new(entries);
    let result = expander.expand("brb");

    assert_eq!(result.expanded_text, "be right back");
    assert_eq!(result.should_press_enter, false);
}

#[test]
fn test_expand_with_auto_enter() {
    let entries = vec![DictionaryEntry {
        id: "1".to_string(),
        trigger: "sig".to_string(),
        expansion: "Best regards, Michael".to_string(),
        suffix: None,
        auto_enter: true,
    }];

    let expander = DictionaryExpander::new(entries);
    let result = expander.expand("sig");

    assert_eq!(result.expanded_text, "Best regards, Michael");
    assert_eq!(result.should_press_enter, true);
}

#[test]
fn test_expand_multiple_entries_one_auto_enter() {
    let entries = vec![
        DictionaryEntry {
            id: "1".to_string(),
            trigger: "brb".to_string(),
            expansion: "be right back".to_string(),
            suffix: None,
            auto_enter: false,
        },
        DictionaryEntry {
            id: "2".to_string(),
            trigger: "sig".to_string(),
            expansion: "Best regards".to_string(),
            suffix: None,
            auto_enter: true,
        },
    ];

    let expander = DictionaryExpander::new(entries);
    let result = expander.expand("brb sig");

    assert_eq!(result.expanded_text, "be right back Best regards");
    assert_eq!(result.should_press_enter, true);  // sig has auto_enter
}

#[test]
fn test_expand_no_match_returns_false() {
    let entries = vec![DictionaryEntry {
        id: "1".to_string(),
        trigger: "brb".to_string(),
        expansion: "be right back".to_string(),
        suffix: None,
        auto_enter: true,
    }];

    let expander = DictionaryExpander::new(entries);
    let result = expander.expand("hello world");

    assert_eq!(result.expanded_text, "hello world");
    assert_eq!(result.should_press_enter, false);  // No match, no auto_enter
}
```

## Related Specs

- [data-model-update.spec.md](./data-model-update.spec.md) - Provides DictionaryEntry with new fields
- [backend-storage-update.spec.md](./backend-storage-update.spec.md) - DictionaryStore provides entries
- [keyboard-simulation.spec.md](./keyboard-simulation.spec.md) - Uses should_press_enter result

## Integration Points

- Production call site: `src-tauri/src/transcription/service.rs` - RecordingTranscriptionService calls expand()
- Connects to: DictionaryStore (source of entries), TranscriptionService (consumer)

## Integration Test

- Test location: `src-tauri/src/dictionary/expander_test.rs`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-22
**Reviewer:** Claude

### Pre-Review Gate Results

```
Build Warning Check:
warning: method `get` is never used (src/dictionary/store.rs:218)
  - This is a PRE-EXISTING issue in DictionaryStore, NOT introduced by this spec

Command Registration Check: PASS (no new commands added)
Event Subscription Check: N/A (no new events)
```

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `expand()` returns `ExpansionResult` struct with `expanded_text` and `should_press_enter` fields | PASS | src-tauri/src/dictionary/expander.rs:9-15, 59-84 |
| When entry has suffix, expansion includes suffix (e.g., "brb" with suffix "." -> "be right back.") | PASS | expander.rs:66-69, test at expander_test.rs:116-124 |
| When entry has no suffix, expansion is unchanged | PASS | expander.rs:68, test at expander_test.rs:126-135 |
| When any expanded entry has `auto_enter: true`, result has `should_press_enter: true` | PASS | expander.rs:74-76, test at expander_test.rs:137-146 |
| Multiple expansions in same text all apply their suffixes correctly | PASS | test at expander_test.rs:148-160 |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Expand "brb" with suffix "." -> "be right back." | PASS | src-tauri/src/dictionary/expander_test.rs:116-124 |
| Expand "brb" without suffix -> "be right back" | PASS | src-tauri/src/dictionary/expander_test.rs:126-135 |
| Expand "brb" with auto_enter=true -> should_press_enter is true | PASS | src-tauri/src/dictionary/expander_test.rs:137-146 |
| Expand text with multiple triggers, one has auto_enter -> should_press_enter is true | PASS | src-tauri/src/dictionary/expander_test.rs:148-160 |
| Expand text with no matches -> should_press_enter is false | PASS | src-tauri/src/dictionary/expander_test.rs:162-171 |

### Manual Review Questions

#### 1. Is the code wired up end-to-end?
- [x] New functions are called from production code (not just tests)
- [x] New structs are instantiated in production code (not just tests)

**Evidence:**
- `ExpansionResult` struct is used in production at `src-tauri/src/transcription/service.rs:316-342`
- `expander.expand()` is called in production transcription flow at `service.rs:319`
- The `expanded_text` field is used for clipboard/paste at `service.rs:343, 352, 366`

#### 2. What would break if this code was deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| ExpansionResult | struct | transcription/service.rs:316-342 | YES |
| ExpansionResult.expanded_text | field | service.rs:343, 352, 366 | YES |
| ExpansionResult.should_press_enter | field | service.rs (returned but not yet consumed) | DEFERRED |
| suffix handling in expand() | logic | service.rs:319 via expander.expand() | YES |
| auto_enter tracking in expand() | logic | service.rs:319 via expander.expand() | YES |

**Note on should_press_enter:** This field is correctly returned but consumption is intentionally deferred to the `keyboard-simulation.spec.md` which is a documented pending spec in this same feature. This is acceptable as the spec explicitly states this is for "the keyboard simulation spec" in its description.

#### 3. Where does the data flow?

```
[Recording stopped]
     |
     v
[TranscriptionService] src-tauri/src/transcription/service.rs
     | calls expander.expand(&text) at line 319
     v
[DictionaryExpander.expand()] src-tauri/src/dictionary/expander.rs:59-84
     | applies suffix if present, tracks auto_enter
     v
[ExpansionResult] { expanded_text, should_press_enter }
     |
     v
[expanded_text used] service.rs:343, 352, 366
     | copied to clipboard and pasted
     v
[should_press_enter] (deferred to keyboard-simulation spec)
```

**Data flow is complete for this spec's scope.** The `should_press_enter` consumption is explicitly deferred to `keyboard-simulation.spec.md`.

#### 4. Are there any deferrals?

| Deferral Text | Location | Tracking Spec |
|---------------|----------|---------------|
| should_press_enter not consumed | service.rs | keyboard-simulation.spec.md |

The spec explicitly documents this in the Description: "Track whether any expanded entry has `auto_enter: true` for the keyboard simulation spec"

#### 5. Automated check results

```
Build Check: PASS (1 pre-existing warning unrelated to this spec)
Command Registration Check: PASS (no new commands)
Event Subscription Check: N/A (no new events)
Test Execution: PASS (27 dictionary tests pass, including 6 new suffix/auto_enter tests)
```

### Code Quality

**Strengths:**
- Clean implementation of ExpansionResult struct with clear documentation
- Proper handling of suffix appending with Option type
- Good test coverage for all spec requirements (6 dedicated tests)
- Production integration in TranscriptionService is complete for expanded_text
- Follows existing code patterns in the codebase

**Concerns:**
- None identified. The should_press_enter field is correctly implemented and available for the subsequent keyboard-simulation spec.

### Verdict

**APPROVED** - All acceptance criteria are met and verified with tests. The ExpansionResult struct is correctly integrated into the production transcription flow. The `should_press_enter` field is intentionally deferred to the documented `keyboard-simulation.spec.md` pending spec, which is acceptable per the spec's description.
