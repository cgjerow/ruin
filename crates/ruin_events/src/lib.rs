pub enum Event {
    Window(WindowEvent),
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
}

pub enum WindowEvent {
    Resized(u32, u32),
    CloseRequested,
    Focused(bool),
}

pub enum KeyboardEvent {
    KeyDown(KeyCode),
    KeyUp(KeyCode),
    CharInput(char),
}

pub enum MouseEvent {
    ButtonDown(MouseButton),
    ButtonUp(MouseButton),
    Moved { x: f32, y: f32 },
    Wheel { delta_x: f32, delta_y: f32 },
}
