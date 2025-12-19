---
status: pending
created: 2025-12-19
completed: null
dependencies:
  - cgeventtap-core
---

# Spec: Media key capture via NSSystemDefined events

## Description

Add media key capture to the CGEventTap implementation. Media keys (volume, brightness, play/pause) are sent as NSSystemDefined events with a specific subtype, not as regular keyboard events. This extends cgeventtap-core to also listen for these events.

## Acceptance Criteria

- [ ] CGEventTap also listens for NSSystemDefined events (type 14)
- [ ] Media key events detected by checking subtype == 8 (NX_SUBTYPE_AUX_CONTROL_BUTTONS)
- [ ] Key code extracted from event data: `(data1 & 0xFFFF0000) >> 16`
- [ ] Key state extracted: `(data1 & 0x0000FF00) >> 8` (0=up, 1=down, 2=repeat)
- [ ] Media keys mapped to human-readable names
- [ ] CapturedKeyEvent emitted with is_media_key=true

## Test Cases

- [ ] Press Volume Up → callback with key_name="VolumeUp", is_media_key=true
- [ ] Press Volume Down → callback with key_name="VolumeDown", is_media_key=true
- [ ] Press Mute → callback with key_name="Mute", is_media_key=true
- [ ] Press Brightness Up → callback with key_name="BrightnessUp", is_media_key=true
- [ ] Press Play/Pause → callback with key_name="PlayPause", is_media_key=true
- [ ] Media key + modifier → callback with both media key and modifier flags

## Dependencies

- cgeventtap-core - base CGEventTap implementation

## Preconditions

- cgeventtap-core spec completed
- Accessibility permission granted

## Implementation Notes

### Event Mask Extension
```rust
// Add NSSystemDefined to the event mask
let event_mask = (1 << CGEventType::KeyDown as u64)
               | (1 << CGEventType::KeyUp as u64)
               | (1 << CGEventType::FlagsChanged as u64)
               | (1 << 14u64); // NSSystemDefined
```

### Media Key Extraction
```rust
const NX_SUBTYPE_AUX_CONTROL_BUTTONS: i64 = 8;

if event_type == 14 { // NSSystemDefined
    let subtype = event.get_integer_value_field(CGEventField::EventSubtype);
    if subtype == NX_SUBTYPE_AUX_CONTROL_BUTTONS {
        let data1 = event.get_integer_value_field(CGEventField::EventData1);
        let key_code = ((data1 as u64) & 0xFFFF0000) >> 16;
        let key_state = ((data1 as u64) & 0x0000FF00) >> 8;
        let pressed = key_state == 1; // 0=up, 1=down, 2=repeat

        // Map key_code to name
    }
}
```

### Media Key Codes
```rust
const NX_KEYTYPE_SOUND_UP: u32 = 0;
const NX_KEYTYPE_SOUND_DOWN: u32 = 1;
const NX_KEYTYPE_MUTE: u32 = 7;
const NX_KEYTYPE_BRIGHTNESS_UP: u32 = 2;
const NX_KEYTYPE_BRIGHTNESS_DOWN: u32 = 3;
const NX_KEYTYPE_PLAY: u32 = 16;
const NX_KEYTYPE_NEXT: u32 = 17;
const NX_KEYTYPE_PREVIOUS: u32 = 18;
const NX_KEYTYPE_FAST: u32 = 19;
const NX_KEYTYPE_REWIND: u32 = 20;
const NX_KEYTYPE_ILLUMINATION_UP: u32 = 21;
const NX_KEYTYPE_ILLUMINATION_DOWN: u32 = 22;
```

### Key Name Mapping
```rust
fn media_key_to_name(key_code: u32) -> String {
    match key_code {
        0 => "VolumeUp".to_string(),
        1 => "VolumeDown".to_string(),
        7 => "Mute".to_string(),
        2 => "BrightnessUp".to_string(),
        3 => "BrightnessDown".to_string(),
        16 => "PlayPause".to_string(),
        17 => "NextTrack".to_string(),
        18 => "PreviousTrack".to_string(),
        19 => "FastForward".to_string(),
        20 => "Rewind".to_string(),
        21 => "KeyboardBrightnessUp".to_string(),
        22 => "KeyboardBrightnessDown".to_string(),
        _ => format!("MediaKey({})", key_code),
    }
}
```

File location: Integrated into `src-tauri/src/keyboard_capture/cgeventtap.rs`

## Related Specs

- cgeventtap-core.spec.md - base implementation
- replace-iokit-hid.spec.md - integration

## Integration Points

- Production call site: Part of cgeventtap module
- Connects to: CGEventTap callback

## Integration Test

- Test location: Manual testing via integration-test spec
- Verification: [ ] Media keys detected in shortcut recording UI
