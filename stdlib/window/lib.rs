/**
 * Looks intimidating, ik
 *
 * to find the actual code, look for the match statement
 * or just ctrl+f for "std::print" or whatever you want to find
 *
 * there is no official documentation for writing Rusty danda libraries at the time of writing this
 * for more information, please refer to the main repository www.github.com/it-2001/Ruda
 *
 */
extern crate runtime;

use runtime::runtime_error::ErrTypes;
use runtime::runtime_types::*;
use runtime::*;

use runtime::user_data::UserData;
use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::*;

fn call(ctx: &mut Context, id: usize, lib_id: usize) -> Result<Types, runtime_error::ErrTypes> {
    let m = &mut ctx.memory;
    match id {
        // Window::new
        0 => {
            let args = m.args();
            let title = match args[0] {
                Types::Pointer(pos, PointerTypes::String) => m.strings.pool[pos].clone(),
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            let settings = match args[1] {
                Types::Pointer(pos, PointerTypes::UserData) => m.user_data.data[pos].as_mut(),
                _ => Err(ErrTypes::InvalidType(
                    args[1],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let settings = settings.as_any_mut().downcast_mut::<WinBuilder>().unwrap();
            let window = Window::new(
                (settings.width, settings.height),
                &title,
                window::Style::CLOSE,
                settings,
            );
            let ud = ctx.memory.user_data.push(Box::new(window));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // WinBuilder::new
        1 => {
            let ud = ctx.memory.user_data.push(Box::new(WinBuilder::new()));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // WinBuilder::title
        2 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let win_builder = m.user_data.data[ud].as_mut();
            let win_builder = win_builder
                .as_any_mut()
                .downcast_mut::<WinBuilder>()
                .unwrap();
            match arg1 {
                Types::Pointer(pos, PointerTypes::String) => {
                    win_builder.title = m.strings.pool[pos].clone();
                }
                Types::Null => (),
                _ => Err(ErrTypes::InvalidType(
                    arg1,
                    Types::Pointer(0, PointerTypes::String),
                ))?,
            };
            let str_pos = m.strings.from_str(&win_builder.title);
            return Ok(Types::Pointer(str_pos, PointerTypes::String));
        }
        // WinBuilder::width
        3 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let win_builder = m.user_data.data[ud].as_mut();
            let win_builder = win_builder
                .as_any_mut()
                .downcast_mut::<WinBuilder>()
                .unwrap();
            match arg1 {
                Types::Uint(i) => {
                    win_builder.width = i as u32;
                }
                Types::Null => (),
                _ => Err(ErrTypes::InvalidType(arg1, Types::Uint(0)))?,
            };
            return Ok(Types::Uint(win_builder.width as usize));
        }
        // WinBuilder::height
        4 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let win_builder = m.user_data.data[ud].as_mut();
            let win_builder = win_builder
                .as_any_mut()
                .downcast_mut::<WinBuilder>()
                .unwrap();
            match arg1 {
                Types::Uint(i) => {
                    win_builder.height = i as u32;
                }
                Types::Null => (),
                _ => Err(ErrTypes::InvalidType(arg1, Types::Uint(0)))?,
            };
            return Ok(Types::Uint(win_builder.height as usize));
        }
        // Window::clear
        5 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.window.clear(Color::BLACK);
        }
        // Window::display
        6 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.window.display();
        }
        // Window::close
        7 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            window.window.close();
        }
        // Window::width
        8 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            match arg1 {
                Types::Uint(i) => {
                    window.window.set_size((i as u32, window.window.size().x));
                }
                Types::Null => (),
                _ => Err(ErrTypes::InvalidType(arg1, Types::Uint(0)))?,
            };
            return Ok(Types::Uint(window.window.size().x as usize));
        }
        // Window::height
        9 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            match arg1 {
                Types::Uint(i) => {
                    window.window.set_size((window.window.size().x, i as u32));
                }
                Types::Null => (),
                _ => Err(ErrTypes::InvalidType(arg1, Types::Uint(0)))?,
            };
            return Ok(Types::Uint(window.window.size().y as usize));
        }
        // Window::pollEvent
        10 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            let event = match window.window.poll_event() {
                Some(e) => e,
                None => return Ok(Types::Null),
            };
            let ud = ctx.memory.user_data.push(Box::new(Event::new(event)));
            return Ok(Types::Pointer(ud, PointerTypes::UserData));
        }
        // Event::code
        11 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let code = match event.event {
                window::Event::Closed => 0,
                window::Event::Resized { .. } => 1,
                window::Event::LostFocus => 2,
                window::Event::GainedFocus => 3,
                window::Event::TextEntered { .. } => 4,
                window::Event::KeyPressed { .. } => 5,
                window::Event::KeyReleased { .. } => 6,
                window::Event::MouseWheelScrolled { .. } => 7,
                window::Event::MouseButtonPressed { .. } => 8,
                window::Event::MouseButtonReleased { .. } => 9,
                window::Event::MouseMoved { .. } => 10,
                _ => 11,
            };
            return Ok(Types::Uint(code));
        }
        // Event::key
        12 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let key = match event.event {
                window::Event::KeyPressed { code, .. } => code as usize,
                window::Event::KeyReleased { code, .. } => code as usize,
                _ => 0,
            };
            return Ok(Types::Uint(key as usize));
        }
        // Window::fps
        13 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => Err(ErrTypes::InvalidType(
                    args[0],
                    Types::Pointer(0, PointerTypes::UserData),
                ))?,
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            match arg1 {
                Types::Uint(i) => {
                    window.window.set_framerate_limit(i as u32);
                }
                _ => Err(ErrTypes::InvalidType(arg1, Types::Uint(0)))?,
            };
            return Ok(Types::Void);
        }
        // Event::input
        14 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let input = match event.event {
                window::Event::TextEntered { unicode } => unicode as char,
                _ => '\0',
            };
            return Ok(Types::Char(input));
        }
        // Window::title
        15 => {
            let args = m.args();
            let arg1 = args[1];
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let window = m.user_data.data[ud].as_mut();
            let window = window.as_any_mut().downcast_mut::<Window>().unwrap();
            match arg1 {
                Types::Pointer(pos, PointerTypes::String) => {
                    window.window.set_title(&m.strings.pool[pos]);
                }
                _ => {
                    return Err(ErrTypes::InvalidType(
                        arg1,
                        Types::Pointer(0, PointerTypes::String),
                    ))
                }
            };
            return Ok(Types::Void);
        }
        // Event::scan
        16 => {
            let args = m.args();
            let ud = match args[0] {
                Types::Pointer(pos, PointerTypes::UserData) => pos,
                _ => {
                    return Err(ErrTypes::InvalidType(
                        args[0],
                        Types::Pointer(0, PointerTypes::UserData),
                    ))
                }
            };
            let event = m.user_data.data[ud].as_mut();
            let event = event.as_any_mut().downcast_mut::<Event>().unwrap();
            let scan = match event.event {
                window::Event::KeyPressed { scan, .. } => scan as usize,
                _ => 0,
            };
            return Ok(Types::Uint(scan as usize));
        }
        _ => unreachable!("Invalid function id, {}", id),
    }
    return Ok(runtime_types::Types::Void);
}

#[no_mangle]
fn register() -> String {
    r#"
    userdata Window > 0i {
        new (title=reg.g1: string, settings=reg.ptr: WinBuilder) > 0i
        fun width(self=reg.ptr, width=reg.g1: uint?): uint > 8i
        fun height(self=reg.ptr, height=reg.g1: uint?): uint > 9i
        fun fps(self=reg.ptr, fps=reg.g1: uint) > 13i
        fun title(self=reg.ptr, title=reg.g1: string)> 15i

        fun clear(self=reg.ptr) > 5i
        fun display(self=reg.ptr) > 6i
        fun close(self=reg.ptr) > 7i

        fun poll(self=reg.ptr): Event? > 10i
    }

    userdata WinBuilder > 1i {
        new () > 1i

        fun title(self=reg.ptr, title=reg.g1: string?): string > 2i
        fun width(self=reg.ptr, width=reg.g1: uint?): uint > 3i
        fun height(self=reg.ptr, height=reg.g1: uint?): uint > 4i
    }

    userdata Event > 2i {
        fun code(self=reg.ptr): Events > 11i
        fun key(self=reg.ptr): Keys > 12i
        fun input(self=reg.ptr): char > 14i
        fun scan(self=reg.ptr): Scan > 15i
    }

    enum Events > 0i {
        Closed
        Resized
        LostFocus
        GainedFocus
        Input
        KeyPressed
        KeyReleased
        MouseWheelScrolled
        MouseButtonPressed
        MouseButtonReleased
        MouseMoved

        Unknown
        None
    }

    enum Keys > 1i { 
        A B C D E F G H I J K L M N O P Q R S T U V W X Y Z 
        Num0 Num1 Num2 Num3 Num4 Num5 Num6 Num7 Num8 Num9 
        Escape LControl LShift LAlt LSystem RControl RShift RAlt RSystem Menu LBracket RBracket 
        Semicolon Comma Period Quote Slash Backslash Tilde Equal Hyphen Space 
        Enter Backspace Tab PageUp PageDown End Home Insert Delete 
        Add Subtract Multiply Divide Left Right Up Down 
        Numpad0 Numpad1 Numpad2 Numpad3 Numpad4 Numpad5 Numpad6 Numpad7 Numpad8 Numpad9 
        F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F15 
        Pause
    }
    enum Scan > 2i { 
        A B C D E F G H I J K L M N O P Q R S T U V W X Y Z 
        Num1 Num2 Num3 Num4 Num5 Num6 Num7 Num8 Num9 Num0 
        Enter Escape Backspace Tab Space Hyphen Equal LBracket RBracke Backslash 
        Semicolon Apostrophe Grave Comma Period Slash 
        F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F15 F16 F17 F18 F19 F20 F21 F22 F23 F24 
        CapsLock PrintScreen ScrollLock Pause Insert Home PageUp Delete End PageDown 
        Right Left Down Up 
        NumLock NumpadDivide NumpadMultiply NumpadMinus NumpadPlus NumpadEqual NumpadEnter NumpadDecimal 
        Numpad1 Numpad2 Numpad3 Numpad4 Numpad5 Numpad6 Numpad7 Numpad8 Numpad9 Numpad 
        NonUsBackslash Application Execute ModeChange Help Menu 
        Select Redo Undo Cut Copy Paste 
        VolumeMute VolumeUp VolumeDown MediaPlayPause 
        MediaStop MediaNextTrack MediaPreviousTrack 
        LControl LShift LAlt LSystem RControl RShift RAlt RSystem 
        Back Forward Refresh Stop Search Favorites HomePage 
        LaunchApplication1 LaunchApplication2 LaunchMail LaunchMediaSelect 
        ScancodeCount 
    }
    "#.to_string()
}

#[no_mangle]
pub fn init(
    _ctx: &mut Context,
    my_id: usize,
) -> fn(&mut Context, usize, usize) -> Result<Types, runtime_error::ErrTypes> {
    call
}

struct Window {
    window: RenderWindow,
    name: String,
    id: usize,
    lib_id: usize,
    gc_method: user_data::GcMethod,
}
impl Window {
    fn new(size: (u32, u32), title: &str, style: window::Style, settings: &WinBuilder) -> Self {
        let window = RenderWindow::new(size, title, style, &Default::default());
        Self {
            window,
            name: "Window".to_string(),
            id: 0,
            lib_id: 0,
            gc_method: user_data::GcMethod::None,
        }
    }
}
impl UserData for Window {
    fn label(&self) -> &str {
        &self.name
    }

    fn id(&self) -> usize {
        self.id
    }

    fn lib_id(&self) -> usize {
        self.lib_id
    }

    fn gc_method(&self) -> &user_data::GcMethod {
        &self.gc_method
    }

    fn cleanup(&mut self) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct WinBuilder {
    title: String,
    width: u32,
    height: u32,
    name: String,
    id: usize,
    lib_id: usize,
    gc_method: user_data::GcMethod,
}
impl WinBuilder {
    fn new() -> Self {
        Self {
            title: "Window".to_string(),
            width: 800,
            height: 600,
            name: "WinBuilder".to_string(),
            id: 1,
            lib_id: 0,
            gc_method: user_data::GcMethod::None,
        }
    }
}
impl UserData for WinBuilder {
    fn label(&self) -> &str {
        &self.name
    }

    fn id(&self) -> usize {
        self.id
    }

    fn lib_id(&self) -> usize {
        self.lib_id
    }

    fn gc_method(&self) -> &user_data::GcMethod {
        &self.gc_method
    }

    fn cleanup(&mut self) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct Event {
    name: String,
    id: usize,
    lib_id: usize,
    gc_method: user_data::GcMethod,
    event: window::Event,
}
impl Event {
    fn new(event: window::Event) -> Self {
        Self {
            name: "Event".to_string(),
            id: 2,
            lib_id: 0,
            gc_method: user_data::GcMethod::None,
            event,
        }
    }
}
impl UserData for Event {
    fn label(&self) -> &str {
        &self.name
    }

    fn id(&self) -> usize {
        self.id
    }

    fn lib_id(&self) -> usize {
        self.lib_id
    }

    fn gc_method(&self) -> &user_data::GcMethod {
        &self.gc_method
    }

    fn cleanup(&mut self) {}
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
