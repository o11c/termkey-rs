#![allow(non_camel_case_types)]

pub use libc::c_char;
pub use libc::c_int;
pub use libc::c_long;
pub use libc::c_ulong;
pub use libc::size_t;

//mod bitset_macro;

pub static TERMKEY_VERSION_MAJOR: c_int = 0;
pub static TERMKEY_VERSION_MINOR: c_int = 17;
#[allow(non_snake_case_functions)]
pub unsafe fn TERMKEY_CHECK_VERSION()
{
    termkey_check_version(TERMKEY_VERSION_MAJOR, TERMKEY_VERSION_MINOR);
}

#[repr(C)] #[deriving(PartialEq, PartialOrd)]
pub enum TermKeySym
{
  TERMKEY_SYM_UNKNOWN = -1,
  TERMKEY_SYM_NONE = 0,

  /* Special names in C0 */
  TERMKEY_SYM_BACKSPACE,
  TERMKEY_SYM_TAB,
  TERMKEY_SYM_ENTER,
  TERMKEY_SYM_ESCAPE,

  /* Special names in G0 */
  TERMKEY_SYM_SPACE,
  TERMKEY_SYM_DEL,

  /* Special keys */
  TERMKEY_SYM_UP,
  TERMKEY_SYM_DOWN,
  TERMKEY_SYM_LEFT,
  TERMKEY_SYM_RIGHT,
  TERMKEY_SYM_BEGIN,
  TERMKEY_SYM_FIND,
  TERMKEY_SYM_INSERT,
  TERMKEY_SYM_DELETE,
  TERMKEY_SYM_SELECT,
  TERMKEY_SYM_PAGEUP,
  TERMKEY_SYM_PAGEDOWN,
  TERMKEY_SYM_HOME,
  TERMKEY_SYM_END,

  /* Special keys from terminfo */
  TERMKEY_SYM_CANCEL,
  TERMKEY_SYM_CLEAR,
  TERMKEY_SYM_CLOSE,
  TERMKEY_SYM_COMMAND,
  TERMKEY_SYM_COPY,
  TERMKEY_SYM_EXIT,
  TERMKEY_SYM_HELP,
  TERMKEY_SYM_MARK,
  TERMKEY_SYM_MESSAGE,
  TERMKEY_SYM_MOVE,
  TERMKEY_SYM_OPEN,
  TERMKEY_SYM_OPTIONS,
  TERMKEY_SYM_PRINT,
  TERMKEY_SYM_REDO,
  TERMKEY_SYM_REFERENCE,
  TERMKEY_SYM_REFRESH,
  TERMKEY_SYM_REPLACE,
  TERMKEY_SYM_RESTART,
  TERMKEY_SYM_RESUME,
  TERMKEY_SYM_SAVE,
  TERMKEY_SYM_SUSPEND,
  TERMKEY_SYM_UNDO,

  /* Numeric keypad special keys */
  TERMKEY_SYM_KP0,
  TERMKEY_SYM_KP1,
  TERMKEY_SYM_KP2,
  TERMKEY_SYM_KP3,
  TERMKEY_SYM_KP4,
  TERMKEY_SYM_KP5,
  TERMKEY_SYM_KP6,
  TERMKEY_SYM_KP7,
  TERMKEY_SYM_KP8,
  TERMKEY_SYM_KP9,
  TERMKEY_SYM_KPENTER,
  TERMKEY_SYM_KPPLUS,
  TERMKEY_SYM_KPMINUS,
  TERMKEY_SYM_KPMULT,
  TERMKEY_SYM_KPDIV,
  TERMKEY_SYM_KPCOMMA,
  TERMKEY_SYM_KPPERIOD,
  TERMKEY_SYM_KPEQUALS,

  /* et cetera ad nauseum */
  TERMKEY_N_SYMS
}

impl ::std::fmt::Show for TermKeySym
{
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
    {
        let symi: c_int = unsafe { ::std::mem::transmute(*self) };
        let _ = write!(fmt, "{}", symi);
        Ok(())
    }
}

#[repr(C)]
pub enum TermKeyType
{
  TERMKEY_TYPE_UNICODE,
  TERMKEY_TYPE_FUNCTION,
  TERMKEY_TYPE_KEYSYM,
  TERMKEY_TYPE_MOUSE,
  TERMKEY_TYPE_POSITION,
  TERMKEY_TYPE_MODEREPORT,
  /* add other recognised types here */

  TERMKEY_TYPE_UNKNOWN_CSI = -1
}

#[repr(C)] #[deriving(PartialEq)]
pub enum TermKeyResult
{
  TERMKEY_RES_NONE,
  TERMKEY_RES_KEY,
  TERMKEY_RES_EOF,
  TERMKEY_RES_AGAIN,
  TERMKEY_RES_ERROR
}

#[repr(C)] #[deriving(PartialEq, PartialOrd)]
pub enum TermKeyMouseEvent
{
  TERMKEY_MOUSE_UNKNOWN,
  TERMKEY_MOUSE_PRESS,
  TERMKEY_MOUSE_DRAG,
  TERMKEY_MOUSE_RELEASE
}

impl ::std::fmt::Show for TermKeyMouseEvent
{
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
    {
        let symi: c_int = unsafe { ::std::mem::transmute(*self) };
        let _ = write!(fmt, "{}", symi);
        Ok(())
    }
}

bitset!(X_TermKey_KeyMod: c_int
{
  TERMKEY_KEYMOD_SHIFT = 1 << 0,
  TERMKEY_KEYMOD_ALT   = 1 << 1,
  TERMKEY_KEYMOD_CTRL  = 1 << 2
})

impl ::std::fmt::Show for X_TermKey_KeyMod
{
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
    {
        let symi: c_int = unsafe { ::std::mem::transmute(*self) };
        let _ = write!(fmt, "{}", symi);
        Ok(())
    }
}

#[repr(C)]
pub struct TermKeyKey
{
  pub type_: TermKeyType,
  pub code: c_long,
  /*
  union {
    long       codepoint; /* TERMKEY_TYPE_UNICODE */
    int        number;    /* TERMKEY_TYPE_FUNCTION */
    TermKeySym sym;       /* TERMKEY_TYPE_KEYSYM */
    char       mouse[4];  /* TERMKEY_TYPE_MOUSE */
                          /* opaque. see termkey_interpret_mouse */
  } code;
  */

  pub modifiers: c_int,

  /* Any Unicode character can be UTF-8 encoded in no more than 6 bytes, plus
   * terminating NUL */
  pub utf8: [c_char, ..7],
}
impl ::std::default::Default for TermKeyKey
{
    fn default() -> TermKeyKey
    {
        TermKeyKey{type_: TERMKEY_TYPE_UNICODE, code: 0, modifiers: 0, utf8: [0, ..7]}
    }
}
impl TermKeyKey
{
    pub unsafe fn codepoint(&self) -> c_long
    {
        self.code
    }
    pub unsafe fn num(&self) -> c_int
    {
        let s: &c_int = ::std::mem::transmute(&self.code);
        *s
    }
    pub unsafe fn sym(&self) -> TermKeySym
    {
        let s: &TermKeySym = ::std::mem::transmute(&self.code);
        *s
    }
}
impl TermKeyKey
{
    pub fn from_codepoint(mods: X_TermKey_KeyMod, codepoint: char, utf8: [c_char, ..7]) -> TermKeyKey
    {
        unsafe
        {
            let mods: c_int = ::std::mem::transmute(mods);
            let codepoint: c_long = codepoint as c_long;
            TermKeyKey{type_: TERMKEY_TYPE_UNICODE, code: codepoint, modifiers: mods, utf8: utf8}
        }
    }
    pub fn from_num(mods: X_TermKey_KeyMod, num: int) -> TermKeyKey
    {
        unsafe
        {
            let mods: c_int = ::std::mem::transmute(mods);
            let num: c_int = num as c_int;
            let mut key = TermKeyKey{type_: TERMKEY_TYPE_FUNCTION, code: 0, modifiers: mods, utf8: [0, ..7]};
            let code: &mut c_int = ::std::mem::transmute(&mut key.code);
            *code = num;
            key
        }
    }
    pub fn from_sym(mods: X_TermKey_KeyMod, sym: TermKeySym) -> TermKeyKey
    {
        unsafe
        {
            let mods: c_int = ::std::mem::transmute(mods);
            let mut key = TermKeyKey{type_: TERMKEY_TYPE_KEYSYM, code: 0, modifiers: mods, utf8: [0, ..7]};
            let code: &mut TermKeySym = ::std::mem::transmute(&mut key.code);
            *code = sym;
            key
        }
    }
    pub fn from_mouse(tk: *mut TermKey, mods: X_TermKey_KeyMod, ev: TermKeyMouseEvent, button: c_int, line: c_int, col: c_int) -> TermKeyKey
    {
        unsafe
        {
            let mods: c_int = ::std::mem::transmute(mods);
            let mut key = TermKeyKey{type_: TERMKEY_TYPE_UNICODE, code: 0, modifiers: mods, utf8: [0, ..7]};
            termkey_construct_mouse(tk, &mut key, ev, button, line, col);
            key
        }
    }
    pub fn from_position(tk: *mut TermKey, line: c_int, col: c_int) -> TermKeyKey
    {
        unsafe
        {
            let mut key = TermKeyKey{type_: TERMKEY_TYPE_UNICODE, code: 0, modifiers: 0, utf8: [0, ..7]};
            termkey_construct_position(tk, &mut key, line, col);
            key
        }
    }
    pub fn from_mode_report(tk: *mut TermKey, initial: c_int, mode: c_int, value: c_int) -> TermKeyKey
    {
        unsafe
        {
            let mut key = TermKeyKey{type_: TERMKEY_TYPE_UNICODE, code: 0, modifiers: 0, utf8: [0, ..7]};
            termkey_construct_modereport(tk, &mut key, initial, mode, value);
            key
        }
    }
}

pub enum TermKey {}

bitset!(X_TermKey_Flag : c_int
{
  TERMKEY_FLAG_NOINTERPRET = 1 << 0, /* Do not interpret C0//DEL codes if possible */
  TERMKEY_FLAG_CONVERTKP   = 1 << 1, /* Convert KP codes to regular keypresses */
  TERMKEY_FLAG_RAW         = 1 << 2, /* Input is raw bytes, not UTF-8 */
  TERMKEY_FLAG_UTF8        = 1 << 3, /* Input is definitely UTF-8 */
  TERMKEY_FLAG_NOTERMIOS   = 1 << 4, /* Do not make initial termios calls on construction */
  TERMKEY_FLAG_SPACESYMBOL = 1 << 5, /* Sets TERMKEY_CANON_SPACESYMBOL */
  TERMKEY_FLAG_CTRLC       = 1 << 6, /* Allow Ctrl-C to be read as normal, disabling SIGINT */
  TERMKEY_FLAG_EINTR       = 1 << 7  /* Return ERROR on signal (EINTR) rather than retry */
})

bitset!(X_TermKey_Canon : c_int
{
  TERMKEY_CANON_SPACESYMBOL = 1 << 0, /* Space is symbolic rather than Unicode */
  TERMKEY_CANON_DELBS       = 1 << 1  /* Del is converted to Backspace */
})

bitset!(TermKeyFormat : c_int
{
  TERMKEY_FORMAT_LONGMOD     = 1 << 0, /* Shift-... instead of S-... */
  TERMKEY_FORMAT_CARETCTRL   = 1 << 1, /* ^X instead of C-X */
  TERMKEY_FORMAT_ALTISMETA   = 1 << 2, /* Meta- or M- instead of Alt- or A- */
  TERMKEY_FORMAT_WRAPBRACKET = 1 << 3, /* Wrap special keys in brackets like <Escape> */
  TERMKEY_FORMAT_SPACEMOD    = 1 << 4, /* M Foo instead of M-Foo */
  TERMKEY_FORMAT_LOWERMOD    = 1 << 5, /* meta or m instead of Meta or M */
  TERMKEY_FORMAT_LOWERSPACE  = 1 << 6, /* page down instead of PageDown */

  TERMKEY_FORMAT_MOUSE_POS   = 1 << 8, /* Include mouse position if relevant; @ col,line */

/* Some useful combinations */
  TERMKEY_FORMAT_VIM         = (TERMKEY_FORMAT_ALTISMETA.bits|TERMKEY_FORMAT_WRAPBRACKET.bits),
  TERMKEY_FORMAT_URWID       = (TERMKEY_FORMAT_LONGMOD.bits|TERMKEY_FORMAT_ALTISMETA.bits|
          TERMKEY_FORMAT_LOWERMOD.bits|TERMKEY_FORMAT_SPACEMOD.bits|TERMKEY_FORMAT_LOWERSPACE.bits)
})


// Better to handle in makefile
//#[link(name = "termkey")]
extern
{
pub fn termkey_check_version(major: c_int, minor: c_int);
pub fn termkey_new(fd: c_int, flags: c_int) -> *mut TermKey;
pub fn termkey_new_abstract(term: *const c_char, flags: c_int) -> *mut TermKey;
pub fn termkey_free(tk: *mut TermKey);
pub fn termkey_destroy(tk: *mut TermKey);

pub fn termkey_start(tk: *mut TermKey) -> c_int;
pub fn termkey_stop(tk: *mut TermKey) -> c_int;
pub fn termkey_is_started(tk: *mut TermKey) -> c_int;

pub fn termkey_get_fd(tk: *mut TermKey) -> c_int;

pub fn termkey_get_flags(tk: *mut TermKey) -> c_int;
pub fn termkey_set_flags(tk: *mut TermKey, newflags: c_int);

pub fn termkey_get_waittime(tk: *mut TermKey) -> c_int;
pub fn termkey_set_waittime(tk: *mut TermKey, msec: c_int);

pub fn termkey_get_canonflags(tk: *mut TermKey) -> c_int;
pub fn termkey_set_canonflags(tk: *mut TermKey, cflags: c_int);

pub fn termkey_get_buffer_size(tk: *mut TermKey) -> size_t;
pub fn termkey_set_buffer_size(tk: *mut TermKey, size: size_t) -> c_int;

pub fn termkey_get_buffer_remaining(tk: *mut TermKey) -> size_t;

pub fn termkey_canonicalise(tk: *mut TermKey, key: *mut TermKeyKey);

pub fn termkey_getkey(tk: *mut TermKey, key: *mut TermKeyKey) -> TermKeyResult;
pub fn termkey_getkey_force(tk: *mut TermKey, key: *mut TermKeyKey) -> TermKeyResult;
pub fn termkey_waitkey(tk: *mut TermKey, key: *mut TermKeyKey) -> TermKeyResult;

pub fn termkey_advisereadable(tk: *mut TermKey) -> TermKeyResult;

pub fn termkey_push_bytes(tk: *mut TermKey, bytes: *const c_char, len: size_t) -> size_t;

pub fn termkey_register_keyname(tk: *mut TermKey, sym: TermKeySym, name: *const c_char) -> TermKeySym;
pub fn termkey_get_keyname(tk: *mut TermKey, sym: TermKeySym) -> *const c_char;
pub fn termkey_lookup_keyname(tk: *mut TermKey, str: *const c_char, sym: *mut TermKeySym) -> *const c_char;

pub fn termkey_keyname2sym(tk: *mut TermKey, keyname: *const c_char) -> TermKeySym;

pub fn termkey_interpret_mouse(tk: *mut TermKey, key: *const TermKeyKey, event: *mut TermKeyMouseEvent, button: *mut c_int, line: *mut c_int, col: *mut c_int) -> TermKeyResult;
pub fn termkey_construct_mouse(tk: *mut TermKey, key: *mut TermKeyKey, event: TermKeyMouseEvent, button: c_int, line: c_int, col: c_int);

pub fn termkey_interpret_position(tk: *mut TermKey, key: *const TermKeyKey, line: *mut c_int, col: *mut c_int) -> TermKeyResult;
pub fn termkey_construct_position(tk: *mut TermKey, key: *mut TermKeyKey, line: c_int, col: c_int);

pub fn termkey_interpret_modereport(tk: *mut TermKey, key: *const TermKeyKey, initial: *mut c_int, mode: *mut c_int, value: *mut c_int) -> TermKeyResult;
pub fn termkey_construct_modereport(tk: *mut TermKey, key: *mut TermKeyKey, initial: c_int, mode: c_int, value: c_int);

pub fn termkey_interpret_csi(tk: *mut TermKey, key: *const TermKeyKey, args: *mut c_long, nargs: *mut size_t, cmd: *mut c_ulong) -> TermKeyResult;

pub fn termkey_strfkey(tk: *mut TermKey, buffer: *mut c_char, len: size_t, key: *mut TermKeyKey, format: TermKeyFormat) -> size_t;
pub fn termkey_strpkey(tk: *mut TermKey, str: *const c_char, key: *mut TermKeyKey, format: TermKeyFormat) -> *const c_char;

pub fn termkey_keycmp(tk: *mut TermKey, key1: *const TermKeyKey, key2: *const TermKeyKey) -> c_int;
}
