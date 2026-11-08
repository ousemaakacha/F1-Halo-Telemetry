pub mod rear_view;

use windows::Win32::{Foundation::HWND, UI::Input::KeyboardAndMouse::*};

pub fn parse_hotkey(text: &str) -> Result<(HOT_KEY_MODIFIERS, u32), String> {
    let mut modifiers = MOD_NOREPEAT;
    let mut key = None;
    for token in text.split('+').map(str::trim) {
        match token.to_ascii_uppercase().as_str() {
            "CTRL" | "CONTROL" => modifiers |= MOD_CONTROL,
            "SHIFT" => modifiers |= MOD_SHIFT,
            "ALT" => modifiers |= MOD_ALT,
            "WIN" => modifiers |= MOD_WIN,
            value => {
                if key.is_some() {
                    return Err("Hotkey: specify only one key".into());
                }
                let code = if value.len() == 1 && value.as_bytes()[0].is_ascii_alphanumeric() {
                    value.as_bytes()[0] as u32
                } else if let Some(number) = value
                    .strip_prefix('F')
                    .and_then(|v| v.parse::<u32>().ok())
                    .filter(|v| (1..=24).contains(v))
                {
                    VK_F1.0 as u32 + number - 1
                } else {
                    return Err("Hotkey: use Ctrl/Alt/Shift/Win + letter, number or F1–F24".into());
                };
                key = Some(code);
            }
        }
    }
    if modifiers == MOD_NOREPEAT {
        return Err("Hotkey: at least one modifier is required".into());
    }
    Ok((modifiers, key.ok_or("Hotkey: missing key")?))
}

pub fn parse_hotkeys(text: [&str; 2]) -> Result<[(HOT_KEY_MODIFIERS, u32); 2], String> {
    let keys = [parse_hotkey(text[0])?, parse_hotkey(text[1])?];
    if keys[0] == keys[1] {
        return Err("Edit and HUD visibility hotkeys must be different".into());
    }
    Ok(keys)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HotkeyBinding {
    pub id: i32,
    pub modifiers: HOT_KEY_MODIFIERS,
    pub key: u32,
}

pub fn register_window_hotkeys(
    hwnd: HWND,
    text: [&str; 2],
) -> ([Option<HotkeyBinding>; 2], Vec<String>) {
    register_hotkeys(text, |binding| unsafe {
        RegisterHotKey(Some(hwnd), binding.id, binding.modifiers, binding.key)
            .map_err(|error| error.to_string())
    })
}

pub fn register_hotkeys(
    text: [&str; 2],
    mut register: impl FnMut(HotkeyBinding) -> Result<(), String>,
) -> ([Option<HotkeyBinding>; 2], Vec<String>) {
    let keys = text.map(parse_hotkey);
    let duplicate =
        matches!((&keys[0], &keys[1]), (Ok(edit), Ok(visibility)) if edit == visibility);
    let mut bindings = [None; 2];
    let mut errors = Vec::new();
    for (index, parsed) in keys.into_iter().enumerate() {
        let result = parsed.and_then(|(modifiers, key)| {
            if index == 1 && duplicate {
                return Err("Edit and HUD visibility hotkeys must be different".into());
            }
            let binding = HotkeyBinding {
                id: index as i32 + 1,
                modifiers,
                key,
            };
            register(binding).map_err(|error| {
                let label = if index == 0 { "Edit" } else { "HUD visibility" };
                format!("{label} hotkey is occupied or unavailable: {error}")
            })?;
            Ok(binding)
        });
        match result {
            Ok(binding) => bindings[index] = Some(binding),
            Err(error) => errors.push(error),
        }
    }
    (bindings, errors)
}

pub fn replace_window_hotkeys(
    hwnd: HWND,
    current: [Option<HotkeyBinding>; 2],
    text: [&str; 2],
    apply: impl FnOnce() -> Result<(), String>,
) -> Result<[Option<HotkeyBinding>; 2], String> {
    replace_hotkeys(
        current,
        text,
        |binding| unsafe {
            RegisterHotKey(Some(hwnd), binding.id, binding.modifiers, binding.key)
                .map_err(|error| error.to_string())
        },
        |id| unsafe {
            let _ = UnregisterHotKey(Some(hwnd), id);
        },
        apply,
    )
}

pub fn replace_hotkeys(
    current: [Option<HotkeyBinding>; 2],
    text: [&str; 2],
    mut register: impl FnMut(HotkeyBinding) -> Result<(), String>,
    mut unregister: impl FnMut(i32),
    apply: impl FnOnce() -> Result<(), String>,
) -> Result<[Option<HotkeyBinding>; 2], String> {
    let keys = parse_hotkeys(text)?;
    let mut next = [None, None];
    for (index, (modifiers, key)) in keys.into_iter().enumerate() {
        if let Some(binding) = current
            .iter()
            .flatten()
            .find(|binding| (binding.modifiers, binding.key) == (modifiers, key))
        {
            next[index] = Some(*binding);
            continue;
        }
        let id = (1..=4)
            .find(|id| {
                !current
                    .iter()
                    .chain(next.iter())
                    .flatten()
                    .any(|binding| binding.id == *id)
            })
            .expect("At most four hotkey registrations");
        let binding = HotkeyBinding { id, modifiers, key };
        if let Err(error) = register(binding) {
            release_staged(current, next, &mut unregister);
            let label = if index == 0 { "Edit" } else { "HUD visibility" };
            return Err(format!(
                "{label} hotkey is occupied or unavailable: {error}"
            ));
        }
        next[index] = Some(binding);
    }
    if let Err(error) = apply() {
        release_staged(current, next, &mut unregister);
        return Err(error);
    }
    // Keep old registrations reserved until every fallible apply step succeeds.
    for binding in current
        .into_iter()
        .flatten()
        .filter(|binding| !next.contains(&Some(*binding)))
    {
        unregister(binding.id);
    }
    Ok(next)
}

fn release_staged(
    current: [Option<HotkeyBinding>; 2],
    next: [Option<HotkeyBinding>; 2],
    unregister: &mut impl FnMut(i32),
) {
    for binding in next
        .into_iter()
        .flatten()
        .filter(|binding| !current.contains(&Some(*binding)))
    {
        unregister(binding.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    #[test]
    fn hotkey_validation() {
        assert_eq!(parse_hotkey("Ctrl+Shift+H").unwrap().1, b'H' as u32);
        assert_eq!(parse_hotkey("Alt+F12").unwrap().1, VK_F12.0 as u32);
        assert!(parse_hotkey("H").is_err());
        assert!(parse_hotkey("Ctrl+H+J").is_err());
        assert!(parse_hotkey("Ctrl+").is_err());
    }

    #[test]
    fn hotkey_pair_compares_effective_keys() {
        for alias in [
            " shift + control + h ",
            "H+SHIFT+CTRL",
            "Ctrl+Control+Shift+H",
        ] {
            assert!(parse_hotkeys(["Ctrl+Shift+H", alias]).is_err());
        }
        assert!(parse_hotkeys(["Ctrl+Shift+H", "Ctrl+Shift+O"]).is_ok());
    }

    fn replace(
        registered: &RefCell<Vec<HotkeyBinding>>,
        current: [Option<HotkeyBinding>; 2],
        text: [&str; 2],
        apply: impl FnOnce() -> Result<(), String>,
    ) -> Result<[Option<HotkeyBinding>; 2], String> {
        replace_hotkeys(
            current,
            text,
            |binding| register(registered, binding),
            |id| registered.borrow_mut().retain(|binding| binding.id != id),
            apply,
        )
    }

    fn register(
        registered: &RefCell<Vec<HotkeyBinding>>,
        binding: HotkeyBinding,
    ) -> Result<(), String> {
        let mut registry = registered.borrow_mut();
        if registry
            .iter()
            .any(|old| (old.modifiers, old.key) == (binding.modifiers, binding.key))
        {
            return Err("Shortcut already registered by another application".into());
        }
        registry.push(binding);
        Ok(())
    }

    #[test]
    fn startup_external_conflict_keeps_the_other_shortcut_registered() {
        let text = ["Ctrl+Shift+H", "Ctrl+Shift+O"];
        for occupied in 0..2 {
            let (modifiers, key) = parse_hotkey(text[occupied]).unwrap();
            let external = HotkeyBinding {
                id: 100,
                modifiers,
                key,
            };
            let registry = RefCell::new(vec![external]);
            let (bindings, errors) = register_hotkeys(text, |binding| register(&registry, binding));
            assert!(bindings[occupied].is_none());
            let available = bindings[1 - occupied].unwrap();
            assert_eq!(*registry.borrow(), vec![external, available]);
            assert_eq!(errors.len(), 1);
            assert!(errors[0].contains(if occupied == 0 {
                "Edit hotkey is occupied"
            } else {
                "HUD visibility hotkey is occupied"
            }));
            let failure = replace(&registry, bindings, ["Alt+H", "Alt+O"], || {
                Err("Save failed".into())
            });
            assert!(failure.is_err());
            assert_eq!(*registry.borrow(), vec![external, available]);
        }
    }

    #[test]
    fn startup_rejects_duplicate_aliases_while_keeping_edit_available() {
        let registry = RefCell::new(Vec::new());
        let (bindings, errors) = register_hotkeys(["Ctrl+Shift+H", "shift+control+h"], |binding| {
            register(&registry, binding)
        });
        assert_eq!(*registry.borrow(), vec![bindings[0].unwrap()]);
        assert!(bindings[1].is_none());
        assert_eq!(
            errors,
            vec!["Edit and HUD visibility hotkeys must be different"]
        );
    }

    #[test]
    fn hotkey_external_conflict_preserves_both_previous_bindings() {
        let registry = RefCell::new(Vec::new());
        let old = replace(
            &registry,
            [None; 2],
            ["Ctrl+Shift+H", "Ctrl+Shift+O"],
            || Ok(()),
        )
        .unwrap();
        let external = HotkeyBinding {
            id: 100,
            modifiers: MOD_NOREPEAT | MOD_ALT,
            key: b'P' as u32,
        };
        registry.borrow_mut().push(external);
        let error = replace(&registry, old, ["Alt+H", "Alt+P"], || {
            panic!("Apply must not run")
        })
        .unwrap_err();
        assert!(error.contains("HUD visibility hotkey is occupied"));
        assert_eq!(
            *registry.borrow(),
            vec![old[0].unwrap(), old[1].unwrap(), external]
        );
    }

    #[test]
    fn hotkey_apply_failure_preserves_previous_bindings() {
        let registry = RefCell::new(Vec::new());
        let old = replace(
            &registry,
            [None; 2],
            ["Ctrl+Shift+H", "Ctrl+Shift+O"],
            || Ok(()),
        )
        .unwrap();
        let error = replace(&registry, old, ["Alt+H", "Alt+O"], || {
            Err("Save failed".into())
        })
        .unwrap_err();
        assert_eq!(error, "Save failed");
        assert_eq!(*registry.borrow(), vec![old[0].unwrap(), old[1].unwrap()]);
    }

    #[test]
    fn hotkey_swap_and_aliases_reuse_existing_registrations() {
        let registry = RefCell::new(Vec::new());
        let old = replace(
            &registry,
            [None; 2],
            ["Ctrl+Shift+H", "Ctrl+Shift+O"],
            || Ok(()),
        )
        .unwrap();
        let next = replace_hotkeys(
            old,
            ["o+shift+control", "shift+h+ctrl"],
            |_| panic!("No registration needed"),
            |_| panic!("No release needed"),
            || Ok(()),
        )
        .unwrap();
        assert_eq!(next, [old[1], old[0]]);
    }

    #[test]
    fn hotkey_success_releases_only_replaced_registrations() {
        let registry = RefCell::new(Vec::new());
        let old = replace(
            &registry,
            [None; 2],
            ["Ctrl+Shift+H", "Ctrl+Shift+O"],
            || Ok(()),
        )
        .unwrap();
        let next = replace(&registry, old, ["Alt+H", "control+shift+o"], || {
            assert!(registry.borrow().contains(&old[0].unwrap()));
            assert!(registry.borrow().contains(&old[1].unwrap()));
            Ok(())
        })
        .unwrap();
        assert_eq!(next[1], old[1]);
        assert!(!registry.borrow().contains(&old[0].unwrap()));
        assert!(registry.borrow().contains(&next[0].unwrap()));
        assert_eq!(registry.borrow().len(), 2);
    }
}
