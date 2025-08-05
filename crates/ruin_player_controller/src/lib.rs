pub fn mousebutton_to_str(button: winit::event::MouseButton) -> Option<&'static str> {
    use winit::event::MouseButton::*;
    Some(match button {
        Left => "mouseleft",
        Right => "mouseright",
        Middle => "mousemiddle",
        _ => return None,
    })
}

pub fn keycode_to_str(key: winit::keyboard::KeyCode) -> Option<&'static str> {
    use winit::keyboard::KeyCode::*;
    Some(match key {
        KeyW => "w",
        KeyA => "a",
        KeyS => "s",
        KeyD => "d",
        ArrowUp => "up",
        ArrowDown => "down",
        ArrowLeft => "left",
        ArrowRight => "right",
        Space => "space",
        Enter => "enter",
        Escape => "escape",
        KeyZ => "z",
        KeyX => "x",
        KeyC => "c",
        KeyV => "v",
        Digit0 => "0",
        Digit1 => "1",
        Digit2 => "2",
        Digit3 => "3",
        Digit4 => "4",
        Digit5 => "5",
        Digit6 => "6",
        Digit7 => "7",
        Digit8 => "8",
        Digit9 => "9",
        KeyQ => "q",
        KeyE => "e",
        KeyR => "r",
        KeyF => "f",
        KeyT => "t",
        KeyY => "y",
        KeyU => "u",
        KeyI => "i",
        KeyO => "o",
        KeyP => "p",
        KeyB => "b",
        KeyN => "n",
        KeyM => "m",
        _ => return None, // Unknown or unhandled key
    })
}
