
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::os::raw::{c_int, c_uint, c_char, c_void};

pub type Uint8 = u8;
pub type Uint32 = u32;
pub type Sint32 = i32;
pub type Uint16 = i16;
pub type SDL_Keycode = Sint32;

#[repr(u32)]
#[derive(Copy,Clone)]
pub enum SDL_BlendMode{
    NONE = 0x00000000,
    BLEND = 0x00000001,
    ADD = 0x00000002,
    MOD = 0x00000004
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SDL_Scancode {
    UNKNOWN = 0,
    A = 4,
    B = 5,
    C = 6,
    D = 7,
    E = 8,
    F = 9,
    G = 10,
    H = 11,
    I = 12,
    J = 13,
    K = 14,
    L = 15,
    M = 16,
    N = 17,
    O = 18,
    P = 19,
    Q = 20,
    R = 21,
    S = 22,
    T = 23,
    U = 24,
    V = 25,
    W = 26,
    X = 27,
    Y = 28,
    Z = 29,
    N1 = 30,
    N2 = 31,
    N3 = 32,
    N4 = 33,
    N5 = 34,
    N6 = 35,
    N7 = 36,
    N8 = 37,
    N9 = 38,
    N0 = 39,
    RETURN = 40,
    ESCAPE = 41,
    BACKSPACE = 42,
    TAB = 43,
    SPACE = 44,
    MINUS = 45,
    EQUALS = 46,
    LEFTBRACKET = 47,
    RIGHTBRACKET = 48,
    BACKSLASH = 49,
    NONUSHASH = 50,
    SEMICOLON = 51,
    APOSTROPHE = 52,
    GRAVE = 53,
    COMMA = 54,
    PERIOD = 55,
    SLASH = 56,
    CAPSLOCK = 57,
    F1 = 58,
    F2 = 59,
    F3 = 60,
    F4 = 61,
    F5 = 62,
    F6 = 63,
    F7 = 64,
    F8 = 65,
    F9 = 66,
    F10 = 67,
    F11 = 68,
    F12 = 69,
    PRINTSCREEN = 70,
    SCROLLLOCK = 71,
    PAUSE = 72,
    INSERT = 73,
    HOME = 74,
    PAGEUP = 75,
    DELETE = 76,
    END = 77,
    PAGEDOWN = 78,
    RIGHT = 79,
    LEFT = 80,
    DOWN = 81,
    UP = 82,
    NUMLOCKCLEAR = 83,
    KP_DIVIDE = 84,
    KP_MULTIPLY = 85,
    KP_MINUS = 86,
    KP_PLUS = 87,
    KP_ENTER = 88,
    KP_1 = 89,
    KP_2 = 90,
    KP_3 = 91,
    KP_4 = 92,
    KP_5 = 93,
    KP_6 = 94,
    KP_7 = 95,
    KP_8 = 96,
    KP_9 = 97,
    KP_0 = 98,
    KP_PERIOD = 99,
    NONUSBACKSLASH = 100,
    APPLICATION = 101,
    POWER = 102,
    KP_EQUALS = 103,
    F13 = 104,
    F14 = 105,
    F15 = 106,
    F16 = 107,
    F17 = 108,
    F18 = 109,
    F19 = 110,
    F20 = 111,
    F21 = 112,
    F22 = 113,
    F23 = 114,
    F24 = 115,
    EXECUTE = 116,
    HELP = 117,
    MENU = 118,
    SELECT = 119,
    STOP = 120,
    AGAIN = 121,
    UNDO = 122,
    CUT = 123,
    COPY = 124,
    PASTE = 125,
    FIND = 126,
    MUTE = 127,
    VOLUMEUP = 128,
    VOLUMEDOWN = 129,
    KP_COMMA = 133,
    KP_EQUALSAS400 = 134,
    INTERNATIONAL1 = 135,
    INTERNATIONAL2 = 136,
    INTERNATIONAL3 = 137,
    INTERNATIONAL4 = 138,
    INTERNATIONAL5 = 139,
    INTERNATIONAL6 = 140,
    INTERNATIONAL7 = 141,
    INTERNATIONAL8 = 142,
    INTERNATIONAL9 = 143,
    LANG1 = 144,
    LANG2 = 145,
    LANG3 = 146,
    LANG4 = 147,
    LANG5 = 148,
    LANG6 = 149,
    LANG7 = 150,
    LANG8 = 151,
    LANG9 = 152,
    ALTERASE = 153,
    SYSREQ = 154,
    CANCEL = 155,
    CLEAR = 156,
    PRIOR = 157,
    RETURN2 = 158,
    SEPARATOR = 159,
    OUT = 160,
    OPER = 161,
    CLEARAGAIN = 162,
    CRSEL = 163,
    EXSEL = 164,
    KP_00 = 176,
    KP_000 = 177,
    THOUSANDSSEPARATOR = 178,
    DECIMALSEPARATOR = 179,
    CURRENCYUNIT = 180,
    CURRENCYSUBUNIT = 181,
    KP_LEFTPAREN = 182,
    KP_RIGHTPAREN = 183,
    KP_LEFTBRACE = 184,
    KP_RIGHTBRACE = 185,
    KP_TAB = 186,
    KP_BACKSPACE = 187,
    KP_A = 188,
    KP_B = 189,
    KP_C = 190,
    KP_D = 191,
    KP_E = 192,
    KP_F = 193,
    KP_XOR = 194,
    KP_POWER = 195,
    KP_PERCENT = 196,
    KP_LESS = 197,
    KP_GREATER = 198,
    KP_AMPERSAND = 199,
    KP_DBLAMPERSAND = 200,
    KP_VERTICALBAR = 201,
    KP_DBLVERTICALBAR = 202,
    KP_COLON = 203,
    KP_HASH = 204,
    KP_SPACE = 205,
    KP_AT = 206,
    KP_EXCLAM = 207,
    KP_MEMSTORE = 208,
    KP_MEMRECALL = 209,
    KP_MEMCLEAR = 210,
    KP_MEMADD = 211,
    KP_MEMSUBTRACT = 212,
    KP_MEMMULTIPLY = 213,
    KP_MEMDIVIDE = 214,
    KP_PLUSMINUS = 215,
    KP_CLEAR = 216,
    KP_CLEARENTRY = 217,
    KP_BINARY = 218,
    KP_OCTAL = 219,
    KP_DECIMAL = 220,
    KP_HEXADECIMAL = 221,
    LCTRL = 224,
    LSHIFT = 225,
    LALT = 226,
    LGUI = 227,
    RCTRL = 228,
    RSHIFT = 229,
    RALT = 230,
    RGUI = 231,
    MODE = 257,
    AUDIONEXT = 258,
    AUDIOPREV = 259,
    AUDIOSTOP = 260,
    AUDIOPLAY = 261,
    AUDIOMUTE = 262,
    MEDIASELECT = 263,
    WWW = 264,
    MAIL = 265,
    CALCULATOR = 266,
    COMPUTER = 267,
    AC_SEARCH = 268,
    AC_HOME = 269,
    AC_BACK = 270,
    AC_FORWARD = 271,
    AC_STOP = 272,
    AC_REFRESH = 273,
    AC_BOOKMARKS = 274,
    BRIGHTNESSDOWN = 275,
    BRIGHTNESSUP = 276,
    DISPLAYSWITCH = 277,
    KBDILLUMTOGGLE = 278,
    KBDILLUMDOWN = 279,
    KBDILLUMUP = 280,
    EJECT = 281,
    SLEEP = 282,
    APP1 = 283,
    APP2 = 284,
    AUDIOREWIND = 285,
    AUDIOFASTFORWARD = 286,
    SDL_NUM_SCANCODES = 512
}

#[repr(C)]
#[derive(Clone,Copy)]
pub struct SDL_Keysym {
    pub scancode: SDL_Scancode,
    pub sym: SDL_Keycode,
    pub mode: Uint16,
    pub unused: Uint32,
}

#[repr(C)]
#[derive(Clone,Copy)]
pub struct SDL_KeyboardEvent {
    pub ty: Uint32,
    pub timestamp: Uint32,
    pub window_id: Uint32,
    pub state: Uint8,
    pub repeat: Uint8,
    pub padding2: Uint8,
    pub padding3: Uint8,
    pub keysym: SDL_Keysym
}

#[repr(C)]
pub union SDL_Event {
    pub event_type: Uint32,
    pub key: SDL_KeyboardEvent,
    padding: [u8;56],
    paranoid_padding: [u8;128]
}

pub const SDL_WINDOWPOS_CENTERED_MASK: c_uint = 0x2FFF0000;
pub const SDL_WINDOWPOS_CENTERED: c_uint = SDL_WINDOWPOS_CENTERED_MASK;
pub const SDL_WINDOW_SHOWN: c_uint = 0x00000004;

pub const SDL_RENDERER_SOFTWARE: c_uint = 0x00000001;
pub const SDL_RENDERER_ACCELERATED: c_uint = 0x00000002;

pub const SDL_KEYDOWN: u32 = 0x300;
pub const SDL_KEYUP: u32 = 0x301;

pub const SDL_PIXELFORMAT_RGB24: u32 = 0x17101803;
pub const SDL_PIXELFORMAT_RGBA8888: u32 = 0x16462004;
pub const SDL_TEXTUREACCESS_TARGET: c_int = 2;

#[repr(C)]
pub struct SDL_Window {
    _mem: [u8; 0]
}

#[repr(C)]
pub struct SDL_Renderer {
    _mem: [u8; 0]
}

#[repr(C)]
pub struct SDL_Texture {
    _mem: [u8; 0]
}

#[repr(C)]
pub struct SDL_Rect {
    pub x: c_int, pub y: c_int,
    pub w: c_int, pub h: c_int
}

#[link(name = "SDL2")]
extern "C" {
    // SDL_video.h
    pub fn SDL_CreateWindow(
        title: *const c_char,
        x: c_int, y: c_int, w: c_int, h: c_int,
        flags: Uint32
    ) -> *mut SDL_Window;

    // SDL_render.h
    pub fn SDL_CreateRenderer(
        window: *mut SDL_Window,
        index: c_int,
        flags: Uint32
    ) -> *mut SDL_Renderer;

    pub fn SDL_SetRenderDrawColor(rdr: *mut SDL_Renderer,
        r: Uint8, g: Uint8, b: Uint8, a: Uint8
    );
    pub fn SDL_RenderClear(rdr: *mut SDL_Renderer) -> c_int;
    pub fn SDL_RenderPresent(rdr: *mut SDL_Renderer);
    pub fn SDL_RenderDrawPoint(rdr: *mut SDL_Renderer, x: c_int, y: c_int) -> c_int;
    pub fn SDL_RenderFillRect(rdr: *mut SDL_Renderer, rect: *const SDL_Rect) -> c_int;

    pub fn SDL_Delay(ms: Uint32);
    
    pub fn SDL_PollEvent(event: *mut SDL_Event) -> c_int;
    
    pub fn SDL_Quit();
    pub fn SDL_DestroyWindow(window: *mut SDL_Window);
    pub fn SDL_SetRenderDrawBlendMode(
        rdr: *mut SDL_Renderer, blend_mode: SDL_BlendMode
    ) -> c_int;

    pub fn SDL_RenderReadPixels(rdr: *mut SDL_Renderer,
        rect: *const SDL_Rect, format: Uint32,
        pixels: *mut c_void, picth: c_int
    ) -> c_int;

    pub fn  SDL_CreateTexture(rdr: *mut SDL_Renderer,
        format: Uint32, access: c_int, w: c_int, h: c_int
    ) -> *mut SDL_Texture;
    
    pub fn SDL_SetRenderTarget(rdr: *mut SDL_Renderer,
        texture: *mut SDL_Texture
    ) -> c_int;

    pub fn SDL_RenderCopy(rdr: *mut SDL_Renderer,
        texture: *mut SDL_Texture,
        srcrect: *const SDL_Rect,
        dstrect: *const SDL_Rect
    ) -> c_int;
}

