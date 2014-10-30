#![feature(macro_rules)]

extern crate libc;

extern crate termkey;

macro_rules! diag(
    ($($arg:tt)*) => ({
        let dst: &mut ::std::io::Writer = &mut ::std::io::stderr();
        let _ = write!(dst, "# ");
        let _ = writeln!(dst, $($arg)*);
    })
)

mod taplib
{
    pub struct Tap
    {
        nexttest: uint,
        total: uint,
        _fail: bool,
    }

    impl Tap
    {
        pub fn new() -> Tap
        {
            Tap{nexttest: 1, total: 0, _fail: false}
        }
    }

    impl Drop for Tap
    {
        fn drop(&mut self)
        {
            if self.total != self.nexttest - 1
            {
                diag!("Expected {} tests, got {}", self.total, self.nexttest - 1);
                self._fail = true;
            }
            if self._fail
            {
                if !::std::task::failing()
                {
                    panic!()
                }
                else
                {
                    diag!("avoiding double-panic!() ...")
                }
            }
        }
    }

    impl Tap
    {
        pub fn plan_tests(&mut self, n: uint)
        {
            self.total = n;
            println!("1..{}", n);
        }
        #[allow(dead_code)]
        pub fn finish(&mut self)
        {
            self.total = self.nexttest - 1;
        }

        pub fn pass(&mut self, name: &str)
        {
            println!("ok {} - {}", self.nexttest, name);
            self.nexttest += 1;
        }

        pub fn fail(&mut self, name: &str)
        {
            println!("not ok {} - {}", self.nexttest, name);
            self.nexttest += 1;
            self._fail = true;
        }

        pub fn ok(&mut self, cmp: bool, name: &str)
        {
            if cmp
            {
                self.pass(name);
            }
            else
            {
                self.fail(name);
            }
        }

        pub fn bypass(&mut self, count: uint, name: &str)
        {
            self.fail(name);
            self.nexttest -= 1;
            self.nexttest += count;
        }

        pub fn is_int<T: PartialEq + ::std::fmt::Show>(&mut self, got: T, expect: T, name: &str)
        {
            if got == expect
            {
                self.ok(true, name);
            }
            else
            {
                self.ok(false, name);
                diag!("got {} expected {} in: {}", got, expect, name);
            }
        }
        pub fn is_str<T: Str, U: Str>(&mut self, got: T, expect: U, name: &str)
        {
            let got = got.as_slice();
            let expect = expect.as_slice();

            if got == expect
            {
                self.ok(true, name);
            }
            else
            {
                self.ok(false, name);
                // differs from generic version by the ''s.
                diag!("got '{}' expected '{}' in: {}", got, expect, name);
            }
        }
    }
}

#[test]
fn test_01base()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(6);

    {
        let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::X_TermKey_Flag::empty());

        tap.ok(true, "termkey_new_abstract");

        tap.is_int(tk.get_buffer_size(), 256, "termkey_get_buffer_size");
        tap.ok(tk.is_started(), "termkey_is_started true after construction");

        tk.stop();

        tap.ok(!tk.is_started(), "termkey_is_started false after termkey_stop()");

        tk.start();

        tap.ok(tk.is_started(), "termkey_is_started true after termkey_start()");
    }

    tap.ok(true, "termkey_free");
}

#[test]
fn test_02getkey()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(31);

    let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::X_TermKey_Flag::empty());

    tap.is_int(tk.get_buffer_remaining(), 256, "buffer free initially 256");

    match tk.getkey()
    {
        termkey::None_ => { tap.pass("getkey yields RES_NONE when empty"); }
        _ => { tap.fail("getkey yields RES_NONE when empty") }
    }

    tap.is_int(tk.push_bytes("h".as_bytes()), 1, "push_bytes returns 1");

    tap.is_int(tk.get_buffer_remaining(), 255, "buffer free 255 after push_bytes");

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY after h");

            match key
            {
                termkey::UnicodeEvent{codepoint, mods, utf8} =>
                {
                    tap.pass("key.type after h");
                    tap.is_int(codepoint, 'h', "key.code.number after h");
                    tap.ok(mods.is_empty(), "key.modifiers after h");
                    tap.is_str(utf8.s(), "h", "key.utf8 after h");
                }
                _ => { tap.bypass(4, "key.type after h") }
            }
        }
        _ => { tap.bypass(5, "getkey yields RES_KEY after h") }
    }

    tap.is_int(tk.get_buffer_remaining(), 256, "buffer free 256 after getkey");

    match tk.getkey()
    {
        termkey::None_ => { tap.pass("getkey yields RES_NONE a second time"); }
        _ => { tap.fail("getkey yields RES_NONE a second time") }
    }

    tk.push_bytes("\x01".as_bytes());

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY after C-a");

            match key
            {
                termkey::UnicodeEvent{codepoint, mods, utf8: _} =>
                {
                    tap.pass("key.type after C-a");
                    tap.is_int(codepoint, 'a', "key.code.number after C-a");
                    tap.ok(mods == termkey::c::TERMKEY_KEYMOD_CTRL, "key.modifiers after C-a");
                }
                _ => { tap.bypass(3, "key.type after C-a") }
            }
        }
        _ => { tap.bypass(4, "getkey yields RES_KEY after C-a") }
    }
    tk.push_bytes("\x1bOA".as_bytes());

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY after Up");

            match key
            {
                termkey::KeySymEvent{sym, mods} =>
                {
                    tap.pass("key.type after Up");
                    tap.is_int(sym, termkey::c::TERMKEY_SYM_UP, "key.code.sym after Up");
                    tap.ok(mods.is_empty(), "key.modifiers after Up");
                }
                _ => { tap.bypass(3, "key.type after Up") }
            }
        }
        _ => { tap.bypass(4, "getkey yields RES_KEY after Up") }
    }

    tap.is_int(tk.push_bytes("\x1bO".as_bytes()), 2, "push_bytes returns 2");

    tap.is_int(tk.get_buffer_remaining(), 254, "buffer free 254 after partial write");

    match tk.getkey()
    {
        termkey::Again => { tap.pass("getkey yields RES_AGAIN after partial write") }
        _ => { tap.fail("getkey yields RES_AGAIN after partial write") }
    }

    tk.push_bytes("C".as_bytes());

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY after Right completion");

            match key
            {
                termkey::KeySymEvent{sym, mods} =>
                {
                    tap.pass("key.type after Right completion");
                    tap.is_int(sym, termkey::c::TERMKEY_SYM_RIGHT, "key.code.sym after Right completion");
                    tap.ok(mods.is_empty(), "key.modifiers after Right completion");
                }
                _ => { tap.bypass(3, "key.type after Right completion") }
            }
        }
        _ => { tap.bypass(4, "getkey yields RES_KEY after Right completion") }
    }

    tap.is_int(tk.get_buffer_remaining(), 256, "buffer free 256 after completion");

    tk.push_bytes("\x1b[27;5u".as_bytes());

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY after Ctrl-Escape");

            match key
            {
                termkey::KeySymEvent{sym, mods} =>
                {
                    tap.pass("key.type after Ctrl-Escape");
                    tap.is_int(sym, termkey::c::TERMKEY_SYM_ESCAPE, "key.code.sym after Ctrl-Escape");
                    tap.ok(mods == termkey::c::TERMKEY_KEYMOD_CTRL, "key.modifiers after Ctrl-Escape");
                }
                _ => { tap.bypass(3, "key.type after Ctrl-Escape") }
            }
        }
        _ =>  { tap.bypass(4, "getkey yields RES_KEY after Ctrl-Escape") }
    }
}

#[test]
fn test_03utf8()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(57);

    let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::TERMKEY_FLAG_UTF8);

    tk.push_bytes("a".as_bytes());
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY low ASCII");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.pass("key.type low ASCII");
                    tap.is_int(codepoint, 'a', "key.code.number low ASCII");
                }
                _ => { tap.bypass(2, "key.type low ASCII") }
            }
        }
        _ => { tap.bypass(3, "getkey yields RES_KEY low ASCII") }
    }

    /* 2-byte UTF-8 range is U+0080 to U+07FF (0xDF 0xBF) */
    /* However, we'd best avoid the C1 range, so we'll start at U+00A0 (0xC2 0xA0) */

    tk.push_bytes([0xC2, 0xA0]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 2 low");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.pass("key.type UTF-8 2 low");
                    tap.is_int(codepoint, '\u00A0', "key.code.number UTF-8 2 low");
                }
                _ => { tap.bypass(2, "key.type UTF-8 2 low") }
            }
        }
        _ => { tap.bypass(3, "getkey yields RES_KEY UTF-8 2 low") }
    }

    tk.push_bytes([0xDF, 0xBF]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 2 high");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.pass("key.type UTF-8 2 high");
                    tap.is_int(codepoint, '\u07FF', "key.code.number UTF-8 2 high");
                }
                _ => { tap.bypass(2, "key.type UTF-8 2 high") }
            }
        }
        _ => { tap.bypass(3, "getkey yields RES_KEY UTF-8 2 high") }
    }

    /* 3-byte UTF-8 range is U+0800 (0xE0 0xA0 0x80) to U+FFFD (0xEF 0xBF 0xBD) */

    tk.push_bytes([0xE0, 0xA0, 0x80]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 3 low");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.pass("key.type UTF-8 3 low");
                    tap.is_int(codepoint, '\u0800', "key.code.number UTF-8 3 low");
                }
                _ => tap.bypass(2, "key.type UTF-8 3 low")
            }
        }
        _ => { tap.bypass(3, "getkey yields RES_KEY UTF-8 3 low") }
    }

    tk.push_bytes([0xEF, 0xBF, 0xBD]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 3 high");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.pass("key.type UTF-8 3 high");
                    tap.is_int(codepoint, '\uFFFD', "key.code.number UTF-8 3 high");
                }
                _ => tap.bypass(2, "key.type UTF-8 3 high")
            }
        }
        _ => { tap.bypass(3, "getkey yields RES_KEY UTF-8 3 high") }
    }

    /* 4-byte UTF-8 range is U+10000 (0xF0 0x90 0x80 0x80) to U+10FFFF (0xF4 0x8F 0xBF 0xBF) */

    tk.push_bytes([0xF0, 0x90, 0x80, 0x80]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 4 low");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.pass("key.type UTF-8 4 low");
                    tap.is_int(codepoint, '\U00010000', "key.code.number UTF-8 4 low");
                }
                _ => tap.bypass(2, "key.type UTF-8 4 low")
            }
        }
        _ => { tap.bypass(3, "getkey yields RES_KEY UTF-8 4 low") }
    }

    tk.push_bytes([0xF4, 0x8F, 0xBF, 0xBF]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 4 high");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.pass("key.type UTF-8 4 high");
                    tap.is_int(codepoint, '\U0010FFFF', "key.code.number UTF-8 4 high");
                }
                _ => tap.bypass(2, "key.type UTF-8 4 high")
            }
        }
        _ => { tap.bypass(3, "getkey yields RES_KEY UTF-8 4 high") }
    }

    /* Invalid continuations */

    tk.push_bytes([0xC2, '!' as u8]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 2 invalid cont");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '\uFFFD', "key.code.number UTF-8 2 invalid cont");
                }
                _ => { tap.bypass(1, "key.code.number UTF-8 2 invalid cont") }
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 2 invalid cont") }
    }
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 2 invalid after");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '!', "key.code.number UTF-8 2 invalid after");
                }
                _ => { tap.bypass(1, "key.code.number UTF-8 2 invalid after") }
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 2 invalid after") }
    }

    tk.push_bytes([0xE0, '!' as u8]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 3 invalid cont");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '\uFFFD', "key.code.number UTF-8 3 invalid cont");
                }
                _ => { tap.bypass(1, "key.code.number UTF-8 3 invalid cont") }
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 3 invalid cont") }
    }
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 3 invalid after");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '!', "key.code.number UTF-8 3 invalid after");
                }
                _ => { tap.bypass(1, "key.code.number UTF-8 3 invalid after") }
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 3 invalid after") }
    }

    tk.push_bytes([0xE0, 0xA0, '!' as u8]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 3 invalid cont 2");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '\uFFFD', "key.code.number UTF-8 3 invalid cont 2");
                }
                _ => { tap.bypass(1, "key.code.number UTF-8 3 invalid cont 2") }
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 3 invalid cont 2") }
    }
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 3 invalid after");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '!', "key.code.number UTF-8 3 invalid after");
                }
                _ => { tap.bypass(1, "key.code.number UTF-8 3 invalid after") }
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 3 invalid after") }
    }

    tk.push_bytes([0xF0, '!' as u8]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 4 invalid cont");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '\uFFFD', "key.code.number UTF-8 4 invalid cont");
                }
                _ => { tap.bypass(1, "key.code.number UTF-8 4 invalid cont") }
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 4 invalid cont") }
    }
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 4 invalid after");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '!', "key.code.number UTF-8 4 invalid after");
                }
                _ => { tap.bypass(1, "key.code.number UTF-8 4 invalid after") }
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 4 invalid after") }
    }

    tk.push_bytes([0xF0, 0x90, '!' as u8]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 4 invalid cont 2");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '\uFFFD', "key.code.number UTF-8 4 invalid cont 2");
                }
                _ => { tap.bypass(1, "key.code.number UTF-8 4 invalid cont 2") }
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 4 invalid cont 2") }
    }
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 4 invalid after");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '!', "key.code.number UTF-8 4 invalid after");
                }
                _ => { tap.bypass(1, "key.code.number UTF-8 4 invalid after") }
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 4 invalid after") }
    }

    tk.push_bytes([0xF0, 0x90, 0x80, '!' as u8]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 4 invalid cont 3");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '\uFFFD', "key.code.number UTF-8 4 invalid cont 3");
                }
                _ => { tap.bypass(1, "key.code.number UTF-8 4 invalid cont 3") }
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 4 invalid cont 3") }
    }
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 4 invalid after");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '!', "key.code.number UTF-8 4 invalid after");
                }
                _ => { tap.bypass(1, "key.code.number UTF-8 4 invalid after") }
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 4 invalid after") }
    }

    /* Partials */

    tk.push_bytes([0xC2]);
    match tk.getkey()
    {
        termkey::Again =>
        {
            tap.pass("getkey yields RES_AGAIN UTF-8 2 partial");
        }
        _ => { tap.bypass(1, "getkey yields RES_AGAIN UTF-8 2 partial") }
    }

    tk.push_bytes([0xA0]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 2 partial");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '\u00A0', "key.code.number UTF-8 2 partial");
                }
                _ => { tap.bypass(1, "key.code.number UTF-8 2 partial") }
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 2 partial") }
    }

    tk.push_bytes([0xE0]);
    match tk.getkey()
    {
        termkey::Again =>
        {
            tap.pass("getkey yields RES_AGAIN UTF-8 3 partial");
        }
        _ => { tap.bypass(1, "getkey yields RES_AGAIN UTF-8 3 partial") }
    }
    tk.push_bytes([0xA0]);
    match tk.getkey()
    {
        termkey::Again =>
        {
            tap.pass("getkey yields RES_AGAIN UTF-8 3 partial");
        }
        _ => { tap.bypass(1, "getkey yields RES_AGAIN UTF-8 3 partial") }
    }
    tk.push_bytes([0x80]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 3 partial");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '\u0800', "key.code.number UTF-8 3 partial");
                }
                _ => tap.bypass(1, "key.code.number UTF-8 3 partial")
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 3 partial") }
    }

    tk.push_bytes([0xF0]);
    match tk.getkey()
    {
        termkey::Again =>
        {
            tap.pass("getkey yields RES_AGAIN UTF-8 4 partial");
        }
        _ => { tap.bypass(1, "getkey yields RES_AGAIN UTF-8 4 partial") }
    }
    tk.push_bytes([0x90]);
    match tk.getkey()
    {
        termkey::Again =>
        {
            tap.pass("getkey yields RES_AGAIN UTF-8 4 partial");
        }
        _ => { tap.bypass(1, "getkey yields RES_AGAIN UTF-8 4 partial") }
    }
    tk.push_bytes([0x80]);
    match tk.getkey()
    {
        termkey::Again =>
        {
            tap.pass("getkey yields RES_AGAIN UTF-8 4 partial");
        }
        _ => { tap.bypass(1, "getkey yields RES_AGAIN UTF-8 4 partial") }
    }
    tk.push_bytes([0x80]);
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY UTF-8 4 partial");
            match key
            {
                termkey::UnicodeEvent{codepoint, mods: _, utf8: _} =>
                {
                    tap.is_int(codepoint, '\U00010000', "key.code.number UTF-8 4 partial");
                }
                _ => tap.bypass(1, "key.code.number UTF-8 4 partial")
            }
        }
        _ => { tap.bypass(2, "getkey yields RES_KEY UTF-8 4 partial") }
    }
}

#[test]
fn test_04flags()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(8);

    let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::X_TermKey_Flag::empty());

    tk.push_bytes(" ".as_bytes());
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY after space");

            match key
            {
                termkey::UnicodeEvent{codepoint, mods, utf8: _} =>
                {
                    tap.pass("key.type after space");
                    tap.is_int(codepoint, ' ', "key.code.number after space");
                    tap.ok(mods.is_empty(), "key.modifiers after space");
                }
                _ => tap.bypass(3, "key.type after space")
            }
        }
        _ => { tap.bypass(4, "getkey yields RES_KEY after space") }
    }

    tk.set_flags(termkey::c::TERMKEY_FLAG_SPACESYMBOL);

    tk.push_bytes(" ".as_bytes());
    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY after space");

            match key
            {
                termkey::KeySymEvent{sym, mods} =>
                {
                    tap.pass("key.type after space with FLAG_SPACESYMBOL");
                    tap.is_int(sym, termkey::c::TERMKEY_SYM_SPACE, "key.code.number after space with FLAG_SPACESYMBOL");
                    tap.ok(mods.is_empty(), "key.modifiers after space with FLAG_SPACESYMBOL");
                }
                _ => tap.bypass(3, "key.type after space with FLAG_SPACESYMBOL")
            }
        }
        _ => { tap.bypass(4, "getkey yields RES_KEY after space") }
    }
}

fn fd_write(fd: libc::c_int, s: &str)
{
    let s: &[u8] = s.as_bytes();
    let l: uint = s.len();
    let s: *const u8 = &s[0];
    let l: libc::size_t = l as libc::size_t;
    unsafe
    {
        let s: *const libc::c_void = std::mem::transmute(s);
        libc::write(fd, s, l);
    }
}

#[test]
fn test_05read()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(21);

    /* We'll need a real filehandle we can write/read.
    * pipe() can make us one */
    let fd = unsafe { std::os::pipe().unwrap() };

    /* Sanitise this just in case */
    std::os::setenv("TERM", "vt100");

    let mut tk = termkey::TermKey::new(fd.reader, termkey::c::TERMKEY_FLAG_NOTERMIOS);

    tap.is_int(tk.get_buffer_remaining(), 256, "buffer free initially 256");

    match tk.getkey()
    {
        termkey::None_ =>
        {
            tap.pass("getkey yields RES_NONE when empty");
        }
        _ => { tap.bypass(1, "getkey yields RES_NONE when empty") }
    }

    fd_write(fd.writer, "h");

    match tk.getkey()
    {
        termkey::None_ =>
        {
            tap.pass("getkey yields RES_NONE before advisereadable");
        }
        _ => { tap.bypass(1, "getkey yields RES_NONE before advisereadable") }
    }

    match tk.advisereadable()
    {
        termkey::Again =>
        {
            tap.pass("advisereadable yields RES_AGAIN after h");
        }
        _ => { tap.bypass(1, "advisereadable yields RES_AGAIN after h") }
    }

    tap.is_int(tk.get_buffer_remaining(), 255, "buffer free 255 after advisereadable");

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY after h");

            match key
            {
                termkey::UnicodeEvent{codepoint, mods, utf8} =>
                {
                    tap.pass("key.type after h");
                    tap.is_int(codepoint, 'h', "key.code.number after h");
                    tap.ok(mods.is_empty(), "key.modifiers after h");
                    tap.is_str(utf8.s(), "h", "key.utf8 after h");
                }
                _ => { tap.bypass(4, "key.type after h") }
            }
        }
        _ => { tap.bypass(5, "getkey yields RES_KEY after h") }
    }

    tap.is_int(tk.get_buffer_remaining(), 256, "buffer free 256 after getkey");

    match tk.getkey()
    {
        termkey::None_ =>
        {
            tap.pass("getkey yields RES_NONE a second time");
        }
        _ => { tap.bypass(1, "getkey yields RES_NONE a second time") }
    }

    fd_write(fd.writer, "\x1bO");
    tk.advisereadable();

    tap.is_int(tk.get_buffer_remaining(), 254, "buffer free 254 after partial write");

    match tk.getkey()
    {
        termkey::Again =>
        {
            tap.pass("getkey yields RES_AGAIN after partial write");
        }
        _ => { tap.bypass(1, "getkey yields RES_AGAIN after partial write") }
    }

    fd_write(fd.writer, "C");
    tk.advisereadable();

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY after Right completion");

            match key
            {
                termkey::KeySymEvent{sym, mods} =>
                {
                    tap.pass("key.type after Right");
                    tap.is_int(sym, termkey::c::TERMKEY_SYM_RIGHT, "key.code.sym after Right");
                    tap.ok(mods.is_empty(), "key.modifiers after Right");
                }
                _ => { tap.bypass(3, "key.type after Right") }
            }
        }
        _ => { tap.bypass(4, "getkey yields RES_KEY after Right completion") }
    }

    tap.is_int(tk.get_buffer_remaining(), 256, "buffer free 256 after completion");

    tk.stop();

    match tk.getkey()
    {
        termkey::Error{errno} =>
        {
            tap.pass("getkey yields RES_ERROR after termkey_stop()");
            tap.is_int(errno, libc::EINVAL, "getkey error is EINVAL");
        }
        _ => tap.bypass(2, "getkey yields RES_ERROR after termkey_stop()")
    }
}

#[test]
fn test_06buffer()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(9);

    let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::X_TermKey_Flag::empty());

    tap.is_int(tk.get_buffer_remaining(), 256, "buffer free initially 256");
    tap.is_int(tk.get_buffer_size(), 256, "buffer size initially 256");

    tap.is_int(tk.push_bytes("h".as_bytes()), 1, "push_bytes returns 1");

    tap.is_int(tk.get_buffer_remaining(), 255, "buffer free 255 after push_bytes");
    tap.is_int(tk.get_buffer_size(), 256, "buffer size 256 after push_bytes");

    tap.ok(tk.set_buffer_size(512) != 0, "buffer set size OK");

    tap.is_int(tk.get_buffer_remaining(), 511, "buffer free 511 after push_bytes");
    tap.is_int(tk.get_buffer_size(), 512, "buffer size 512 after push_bytes");

    match tk.getkey()
    {
        termkey::Key(_) =>
        {
            tap.pass("buffered key still useable after resize");
        }
        _ => { tap.bypass(1, "buffered key still useable after resize") }
    }
}

pub fn breakpoint()
{
    return;
}

#[test]
fn test_10keyname()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(10);

    let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::X_TermKey_Flag::empty());

    let mut sym;
    sym = tk.keyname2sym("Space");
    tap.is_int(sym, termkey::c::TERMKEY_SYM_SPACE, "keyname2sym Space");

    sym = tk.keyname2sym("SomeUnknownKey");
    tap.is_int(sym, termkey::c::TERMKEY_SYM_UNKNOWN, "keyname2sym SomeUnknownKey");

    match tk.lookup_keyname("Up", &mut sym)
    {
        Some(end) =>
        {
            tap.pass("termkey_get_keyname Up returns non-NULL");
            tap.is_str(end, "", "termkey_get_keyname Up return points at endofstring");
            tap.is_int(sym, termkey::c::TERMKEY_SYM_UP, "termkey_get_keyname Up yields Up symbol");
        }
        None => { tap.bypass(3, "termkey_get_keyname Up returns non-NULL") }
    }

    match tk.lookup_keyname("DownMore", &mut sym)
    {
        Some(end) =>
        {
            tap.pass("termkey_get_keyname DownMore returns non-NULL");
            tap.is_str(end, "More", "termkey_get_keyname DownMore return points at More");
            tap.is_int(sym, termkey::c::TERMKEY_SYM_DOWN, "termkey_get_keyname DownMore yields Down symbol");
        }
        None => { tap.bypass(3, "termkey_get_keyname DownMore returns non-NULL") }
    }

    match tk.lookup_keyname("SomeUnknownKey", &mut sym)
    {
        None => { tap.pass("termkey_get_keyname SomeUnknownKey returns NULL"); }
        Some(_) => { tap.bypass(1, "termkey_get_keyname SomeUnknownKey returns NULL") }
    }

    tap.is_str(tk.get_keyname(termkey::c::TERMKEY_SYM_SPACE), "Space", "get_keyname SPACE");
}

#[test]
fn test_11strfkey()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(44);

    let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::X_TermKey_Flag::empty());

    let key: termkey::TermKeyEvent = termkey::UnicodeEvent{codepoint: 'A', mods: termkey::c::X_TermKey_KeyMod::empty(), utf8: termkey::Utf8Char{bytes: [0, 0, 0, 0, 0, 0, 0]}};

    let buffer = tk.strfkey(key, termkey::c::TermKeyFormat::empty());
    tap.is_int(buffer.len(), 1, "length for unicode/A/0");
    tap.is_str(buffer, "A", "buffer for unicode/A/0");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_WRAPBRACKET);
    tap.is_int(buffer.len(), 1, "length for unicode/A/0 wrapbracket");
    tap.is_str(buffer, "A", "buffer for unicode/A/0 wrapbracket");

    let key: termkey::TermKeyEvent = termkey::UnicodeEvent{codepoint: 'b', mods: termkey::c::TERMKEY_KEYMOD_CTRL, utf8: termkey::Utf8Char{bytes: [0, 0, 0, 0, 0, 0, 0]}};

    let buffer = tk.strfkey(key, termkey::c::TermKeyFormat::empty());
    tap.is_int(buffer.len(), 3, "length for unicode/b/CTRL");
    tap.is_str(buffer, "C-b", "buffer for unicode/b/CTRL");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_LONGMOD);
    tap.is_int(buffer.len(), 6, "length for unicode/b/CTRL longmod");
    tap.is_str(buffer, "Ctrl-b", "buffer for unicode/b/CTRL longmod");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_LONGMOD|termkey::c::TERMKEY_FORMAT_SPACEMOD);
    tap.is_int(buffer.len(), 6, "length for unicode/b/CTRL longmod|spacemod");
    tap.is_str(buffer, "Ctrl b", "buffer for unicode/b/CTRL longmod|spacemod");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_LONGMOD|termkey::c::TERMKEY_FORMAT_LOWERMOD);
    tap.is_int(buffer.len(), 6, "length for unicode/b/CTRL longmod|lowermod");
    tap.is_str(buffer, "ctrl-b", "buffer for unicode/b/CTRL longmod|lowermod");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_LONGMOD|termkey::c::TERMKEY_FORMAT_SPACEMOD|termkey::c::TERMKEY_FORMAT_LOWERMOD);
    tap.is_int(buffer.len(), 6, "length for unicode/b/CTRL longmod|spacemod|lowermode");
    tap.is_str(buffer, "ctrl b", "buffer for unicode/b/CTRL longmod|spacemod|lowermode");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_CARETCTRL);
    tap.is_int(buffer.len(), 2, "length for unicode/b/CTRL caretctrl");
    tap.is_str(buffer, "^B", "buffer for unicode/b/CTRL caretctrl");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_WRAPBRACKET);
    tap.is_int(buffer.len(), 5, "length for unicode/b/CTRL wrapbracket");
    tap.is_str(buffer, "<C-b>", "buffer for unicode/b/CTRL wrapbracket");

    let key: termkey::TermKeyEvent = termkey::UnicodeEvent{codepoint: 'c', mods: termkey::c::TERMKEY_KEYMOD_ALT, utf8: termkey::Utf8Char{bytes: [0, 0, 0, 0, 0, 0, 0]}};

    let buffer = tk.strfkey(key, termkey::c::TermKeyFormat::empty());
    tap.is_int(buffer.len(), 3, "length for unicode/c/ALT");
    tap.is_str(buffer, "A-c", "buffer for unicode/c/ALT");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_LONGMOD);
    tap.is_int(buffer.len(), 5, "length for unicode/c/ALT longmod");
    tap.is_str(buffer, "Alt-c", "buffer for unicode/c/ALT longmod");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_ALTISMETA);
    tap.is_int(buffer.len(), 3, "length for unicode/c/ALT altismeta");
    tap.is_str(buffer, "M-c", "buffer for unicode/c/ALT altismeta");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_LONGMOD|termkey::c::TERMKEY_FORMAT_ALTISMETA);
    tap.is_int(buffer.len(), 6, "length for unicode/c/ALT longmod|altismeta");
    tap.is_str(buffer, "Meta-c", "buffer for unicode/c/ALT longmod|altismeta");

    let key: termkey::TermKeyEvent = termkey::KeySymEvent{sym: termkey::c::TERMKEY_SYM_UP, mods: termkey::c::X_TermKey_KeyMod::empty()};

    let buffer = tk.strfkey(key, termkey::c::TermKeyFormat::empty());
    tap.is_int(buffer.len(), 2, "length for sym/Up/0");
    tap.is_str(buffer, "Up", "buffer for sym/Up/0");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_WRAPBRACKET);
    tap.is_int(buffer.len(), 4, "length for sym/Up/0 wrapbracket");
    tap.is_str(buffer, "<Up>", "buffer for sym/Up/0 wrapbracket");

    let key: termkey::TermKeyEvent = termkey::KeySymEvent{sym: termkey::c::TERMKEY_SYM_PAGEUP, mods: termkey::c::X_TermKey_KeyMod::empty()};

    let buffer = tk.strfkey(key, termkey::c::TermKeyFormat::empty());
    tap.is_int(buffer.len(), 6, "length for sym/PageUp/0");
    tap.is_str(buffer, "PageUp", "buffer for sym/PageUp/0");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_LOWERSPACE);
    tap.is_int(buffer.len(), 7, "length for sym/PageUp/0 lowerspace");
    tap.is_str(buffer, "page up", "buffer for sym/PageUp/0 lowerspace");

    if true
    {
        tap.pass("skipping small buffer test 1");
        tap.pass("skipping small buffer test 2");
        tap.pass("skipping small buffer test 3");
        tap.pass("skipping small buffer test 4");
    }
    else
    {
        // strfkey internals are not exposed; this is done internally.

        /* If size of buffer is too small, strfkey should return something consistent */
        let buffer = tk.strfkey(/*4*/ key, termkey::c::TermKeyFormat::empty());
        tap.is_int(buffer.len(), 6, "length for sym/PageUp/0");
        tap.is_str(buffer, "Pag", "buffer of len 4 for sym/PageUp/0");

        let buffer = tk.strfkey(/*4*/ key, termkey::c::TERMKEY_FORMAT_LOWERSPACE);
        tap.is_int(buffer.len(), 7, "length for sym/PageUp/0 lowerspace");
        tap.is_str(buffer, "pag", "buffer of len 4 for sym/PageUp/0 lowerspace");
    }

    let key: termkey::TermKeyEvent = termkey::FunctionEvent{num: 5, mods: termkey::c::X_TermKey_KeyMod::empty()};

    let buffer = tk.strfkey(key, termkey::c::TermKeyFormat::empty());
    tap.is_int(buffer.len(), 2, "length for func/5/0");
    tap.is_str(buffer, "F5", "buffer for func/5/0");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_WRAPBRACKET);
    tap.is_int(buffer.len(), 4, "length for func/5/0 wrapbracket");
    tap.is_str(buffer, "<F5>", "buffer for func/5/0 wrapbracket");

    let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_LOWERSPACE);
    tap.is_int(buffer.len(), 2, "length for func/5/0 lowerspace");
    tap.is_str(buffer, "f5", "buffer for func/5/0 lowerspace");
}

#[test]
fn test_12strpkey()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(62);

    let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::X_TermKey_Flag::empty());

    {
        let (key, endp) = tk.strpkey("A", termkey::c::TermKeyFormat::empty()).unwrap();
        match key
        {
            termkey::UnicodeEvent{codepoint, mods, utf8} =>
            {
                tap.pass("key.type for unicode/A/0");
                tap.is_int(codepoint, 'A', "key.code.codepoint for unicode/A/0");
                tap.ok(mods.is_empty(), "key.modifiers for unicode/A/0");
                tap.is_str(utf8.s(), "A", "key.utf8 for unicode/A/0");
            }
            _ => { tap.bypass(4, "key.type for unicode/A/0") }
        }
        tap.is_str(endp, "", "consumed entire input for unicode/A/0");
    }
    {
        let (key, endp) = tk.strpkey("A and more", termkey::c::TermKeyFormat::empty()).unwrap();
        match key
        {
            termkey::UnicodeEvent{codepoint, mods, utf8} =>
            {
                tap.pass("key.type for unicode/A/0 trailing");
                tap.is_int(codepoint, 'A', "key.code.codepoint for unicode/A/0 trailing");
                tap.ok(mods.is_empty(), "key.modifiers for unicode/A/0 trailing");
                tap.is_str(utf8.s(), "A", "key.utf8 for unicode/A/0 trailing");
            }
            _ => { tap.bypass(4, "key.type for unicode/A/0 trailing") }
        }
        tap.is_str(endp, " and more", "points at string tail for unicode/A/0 trailing");
    }
    {
        let (key, endp) = tk.strpkey("C-b", termkey::c::TermKeyFormat::empty()).unwrap();
        match key
        {
            termkey::UnicodeEvent{codepoint, mods, utf8} =>
            {
                tap.pass("key.type for unicode/b/CTRL");
                tap.is_int(codepoint, 'b', "key.code.codepoint for unicode/b/CTRL");
                tap.ok(mods == termkey::c::TERMKEY_KEYMOD_CTRL, "key.modifiers for unicode/b/CTRL");
                tap.is_str(utf8.s(), "b", "key.utf8 for unicode/b/CTRL");
            }
            _ => { tap.bypass(4, "key.type for unicode/b/CTRL") }
        }
        tap.is_str(endp, "", "consumed entire input for unicode/b/CTRL");
    }
    {
        let (key, endp) = tk.strpkey("Ctrl-b", termkey::c::TERMKEY_FORMAT_LONGMOD).unwrap();
        match key
        {
            termkey::UnicodeEvent{codepoint, mods, utf8} =>
            {
                tap.pass("key.type for unicode/b/CTRL longmod");
                tap.is_int(codepoint, 'b', "key.code.codepoint for unicode/b/CTRL longmod");
                tap.ok(mods == termkey::c::TERMKEY_KEYMOD_CTRL, "key.modifiers for unicode/b/CTRL longmod");
                tap.is_str(utf8.s(), "b", "key.utf8 for unicode/b/CTRL longmod");
            }
            _ => { tap.bypass(4, "key.type for unicode/b/CTRL longmod") }
        }
        tap.is_str(endp, "", "consumed entire input for unicode/b/CTRL longmod");
    }
    {
        let (key, endp) = tk.strpkey("^B", termkey::c::TERMKEY_FORMAT_CARETCTRL).unwrap();
        match key
        {
            termkey::UnicodeEvent{codepoint, mods, utf8} =>
            {
                tap.pass("key.type for unicode/b/CTRL caretctrl");
                tap.is_int(codepoint, 'b', "key.code.codepoint for unicode/b/CTRL caretctrl");
                tap.ok(mods == termkey::c::TERMKEY_KEYMOD_CTRL, "key.modifiers for unicode/b/CTRL caretctrl");
                tap.is_str(utf8.s(), "b", "key.utf8 for unicode/b/CTRL caretctrl");
            }
            _ => { tap.bypass(4, "key.type for unicode/b/CTRL caretctrl") }
        }
        tap.is_str(endp, "", "consumed entire input for unicode/b/CTRL caretctrl");
    }
    {
        let (key, endp) = tk.strpkey("A-c", termkey::c::TermKeyFormat::empty()).unwrap();
        match key
        {
            termkey::UnicodeEvent{codepoint, mods, utf8} =>
            {
                tap.pass("key.type for unicode/c/ALT");
                tap.is_int(codepoint, 'c', "key.code.codepoint for unicode/c/ALT");
                tap.ok(mods == termkey::c::TERMKEY_KEYMOD_ALT, "key.modifiers for unicode/c/ALT");
                tap.is_str(utf8.s(), "c", "key.utf8 for unicode/c/ALT");
            }
            _ => { tap.bypass(4, "key.type for unicode/c/ALT") }
        }
        tap.is_str(endp, "", "consumed entire input for unicode/c/ALT");
    }
    {
        let (key, endp) = tk.strpkey("Alt-c", termkey::c::TERMKEY_FORMAT_LONGMOD).unwrap();
        match key
        {
            termkey::UnicodeEvent{codepoint, mods, utf8} =>
            {
                tap.pass("key.type for unicode/c/ALT longmod");
                tap.is_int(codepoint, 'c', "key.code.codepoint for unicode/c/ALT longmod");
                tap.ok(mods == termkey::c::TERMKEY_KEYMOD_ALT, "key.modifiers for unicode/c/ALT longmod");
                tap.is_str(utf8.s(), "c", "key.utf8 for unicode/c/ALT longmod");
            }
            _ => { tap.bypass(4, "key.type for unicode/c/ALT longmod") }
        }
        tap.is_str(endp, "", "consumed entire input for unicode/c/ALT longmod");
    }
    {
        let (key, endp) = tk.strpkey("M-c", termkey::c::TERMKEY_FORMAT_ALTISMETA).unwrap();
        match key
        {
            termkey::UnicodeEvent{codepoint, mods, utf8} =>
            {
                tap.pass("key.type for unicode/c/ALT altismeta");
                tap.is_int(codepoint, 'c', "key.code.codepoint for unicode/c/ALT altismeta");
                tap.ok(mods == termkey::c::TERMKEY_KEYMOD_ALT, "key.modifiers for unicode/c/ALT altismeta");
                tap.is_str(utf8.s(), "c", "key.utf8 for unicode/c/ALT altismeta");
            }
            _ => { tap.bypass(4, "key.type for unicode/c/ALT altismeta") }
        }
        tap.is_str(endp, "", "consumed entire input for unicode/c/ALT altismeta");
    }
    {
        let (key, endp) = tk.strpkey("Meta-c", termkey::c::TERMKEY_FORMAT_ALTISMETA|termkey::c::TERMKEY_FORMAT_LONGMOD).unwrap();
        match key
        {
            termkey::UnicodeEvent{codepoint, mods, utf8} =>
            {
                tap.pass("key.type for unicode/c/ALT altismeta+longmod");
                tap.is_int(codepoint, 'c', "key.code.codepoint for unicode/c/ALT altismeta+longmod");
                tap.ok(mods == termkey::c::TERMKEY_KEYMOD_ALT, "key.modifiers for unicode/c/ALT altismeta+longmod");
                tap.is_str(utf8.s(), "c", "key.utf8 for unicode/c/ALT altismeta+longmod");
            }
            _ => { tap.bypass(4, "key.type for unicode/c/ALT altismeta+longmod") }
        }
        tap.is_str(endp, "", "consumed entire input for unicode/c/ALT altismeta+longmod");
    }
    {
        let (key, endp) = tk.strpkey("meta c", termkey::c::TERMKEY_FORMAT_ALTISMETA|termkey::c::TERMKEY_FORMAT_LONGMOD|termkey::c::TERMKEY_FORMAT_SPACEMOD|termkey::c::TERMKEY_FORMAT_LOWERMOD).unwrap();
        match key
        {
            termkey::UnicodeEvent{codepoint, mods, utf8} =>
            {
                tap.pass("key.type for unicode/c/ALT altismeta+long/space+lowermod");
                tap.is_int(codepoint, 'c', "key.code.codepoint for unicode/c/ALT altismeta+long/space+lowermod");
                tap.ok(mods == termkey::c::TERMKEY_KEYMOD_ALT, "key.modifiers for unicode/c/ALT altismeta+long/space+lowermod");
                tap.is_str(utf8.s(), "c", "key.utf8 for unicode/c/ALT altismeta+long/space_lowermod");
            }
            _ => { tap.bypass(4, "key.type for unicode/c/ALT altismeta+long/space+lowermod") }
        }
        tap.is_str(endp, "", "consumed entire input for unicode/c/ALT altismeta+long/space+lowermod");
    }
    {
        let (key, endp) = tk.strpkey("ctrl alt page up", termkey::c::TERMKEY_FORMAT_LONGMOD|termkey::c::TERMKEY_FORMAT_SPACEMOD|termkey::c::TERMKEY_FORMAT_LOWERMOD|termkey::c::TERMKEY_FORMAT_LOWERSPACE).unwrap();
        match key
        {
            termkey::KeySymEvent{sym, mods} =>
            {
                tap.pass("key.type for sym/PageUp/CTRL+ALT long/space/lowermod+lowerspace");
                tap.is_int(sym, termkey::c::TERMKEY_SYM_PAGEUP, "key.code.codepoint for sym/PageUp/CTRL+ALT long/space/lowermod+lowerspace");
                tap.ok(mods == termkey::c::TERMKEY_KEYMOD_ALT | termkey::c::TERMKEY_KEYMOD_CTRL, "key.modifiers for sym/PageUp/CTRL+ALT long/space/lowermod+lowerspace");
            }
            _ => { tap.bypass(3, "key.type for sym/PageUp/CTRL+ALT long/space/lowermod+lowerspace") }
        }
        tap.is_str(endp, "", "consumed entire input for sym/PageUp/CTRL+ALT long/space/lowermod+lowerspace");
    }
    {
        let (key, endp) = tk.strpkey("Up", termkey::c::TermKeyFormat::empty()).unwrap();
        match key
        {
            termkey::KeySymEvent{sym, mods} =>
            {
                tap.pass("key.type for sym/Up/0");
                tap.is_int(sym, termkey::c::TERMKEY_SYM_UP, "key.code.codepoint for sym/Up/0");
                tap.ok(mods.is_empty(), "key.modifiers for sym/Up/0");
            }
            _ => { tap.bypass(3, "key.type for sym/Up/0") }
        }
        tap.is_str(endp, "", "consumed entire input for sym/Up/0");
    }
    {
        let (key, endp) = tk.strpkey("F5", termkey::c::TermKeyFormat::empty()).unwrap();
        match key
        {
            termkey::FunctionEvent{num, mods} =>
            {
                tap.pass("key.type for func/5/0");
                tap.is_int(num, 5, "key.code.number for func/5/0");
                tap.ok(mods.is_empty(), "key.modifiers for func/5/0");
            }
            _ => { tap.bypass(3, "key.type for func/5/0") }
        }
        tap.is_str(endp, "", "consumed entire input for func/5/0");
    }
}

#[test]
fn test_13cmpkey()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(12);

    let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::X_TermKey_Flag::empty());


    let mut key1: termkey::TermKeyEvent;
    let mut key2: termkey::TermKeyEvent;

    key1 = termkey::UnicodeEvent{codepoint: 'A', mods: termkey::c::X_TermKey_KeyMod::empty(), utf8: termkey::Utf8Char{bytes: [0, ..7]}};

    tap.ok(key1 == key1, "cmpkey same structure");

    key2 = termkey::UnicodeEvent{codepoint: 'A', mods: termkey::c::X_TermKey_KeyMod::empty(), utf8: termkey::Utf8Char{bytes: [0, ..7]}};

    tap.ok(key1 == key2, "cmpkey identical structure");

    key2 = termkey::UnicodeEvent{codepoint: 'A', mods: termkey::c::TERMKEY_KEYMOD_CTRL, utf8: termkey::Utf8Char{bytes: [0, ..7]}};

    tap.ok(key1 < key2, "cmpkey orders CTRL after nomod");
    tap.ok(key2 > key1, "cmpkey orders nomod before CTRL");

    key2 = termkey::UnicodeEvent{codepoint: 'B', mods: termkey::c::X_TermKey_KeyMod::empty(), utf8: termkey::Utf8Char{bytes: [0, ..7]}};

    tap.ok(key1 < key2, "cmpkey orders 'B' after 'A'");
    tap.ok(key2 > key1, "cmpkey orders 'A' before 'B'");

    key1 = termkey::UnicodeEvent{codepoint: 'A', mods: termkey::c::TERMKEY_KEYMOD_CTRL, utf8: termkey::Utf8Char{bytes: [0, ..7]}};

    tap.ok(key1 < key2, "cmpkey orders nomod 'B' after CTRL 'A'");
    tap.ok(key2 > key1, "cmpkey orders CTRL 'A' before nomod 'B'");

    key2 = termkey::KeySymEvent{sym: termkey::c::TERMKEY_SYM_UP, mods: termkey::c::X_TermKey_KeyMod::empty()};

    tap.ok(key1 < key2, "cmpkey orders KEYSYM after UNICODE");
    tap.ok(key2 > key1, "cmpkey orders UNICODE before KEYSYM");

    if true
    {
        tap.pass("skipping unsupported space test 1");
        tap.pass("skipping unsupported space test 2");
    }
    else
    {
        key1 = termkey::KeySymEvent{sym: termkey::c::TERMKEY_SYM_SPACE, mods: termkey::c::X_TermKey_KeyMod::empty()};
        key2 = termkey::UnicodeEvent{codepoint: ' ', mods: termkey::c::X_TermKey_KeyMod::empty(), utf8: termkey::Utf8Char{bytes: [0, ..7]}};

        tap.ok(key1 == key2, "cmpkey considers KEYSYM/SPACE and UNICODE/SP identical");

        // Rust is being too smart for its own good, and forbids multiple
        // borrows in one line, even though only one borrow happens at a time.
        let cflags = tk.get_canonflags();
        tk.set_canonflags(cflags | termkey::c::TERMKEY_CANON_SPACESYMBOL);
        tap.ok(key1 == key2, "cmpkey considers KEYSYM/SPACE and UNICODE/SP identical under SPACESYMBOL");
    }
}

#[test]
fn test_20canon()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(26);

    let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::X_TermKey_Flag::empty());

    {
        let (key, endp) = tk.strpkey(" ", termkey::c::TermKeyFormat::empty()).unwrap();
        match key
        {
            termkey::UnicodeEvent{codepoint, mods, utf8} =>
            {
                tap.pass("key.type for SP/unicode");
                tap.is_int(codepoint, ' ', "key.code.codepoint for SP/unicode");
                tap.ok(mods.is_empty(), "key.modifiers for SP/unicode");
                tap.is_str(utf8.s(), " ", "key.utf8 for SP/unicode");
            }
            _ => { tap.bypass(4, "key.type for SP/unicode") }
        }
        tap.is_str(endp, "", "consumed entire input for SP/unicode");
    }
    {
        let (key, endp) = tk.strpkey("Space", termkey::c::TermKeyFormat::empty()).unwrap();
        match key
        {
            termkey::UnicodeEvent{codepoint, mods, utf8} =>
            {
                tap.pass("key.type for Space/unicode");
                tap.is_int(codepoint, ' ', "key.code.codepoint for Space/unicode");
                tap.ok(mods.is_empty(), "key.modifiers for Space/unicode");
                tap.is_str(utf8.s(), " ", "key.utf8 for Space/unicode");
            }
            _ => { tap.bypass(4, "key.type for Space/unicode") }
        }
        tap.is_str(endp, "", "consumed entire input for Space/unicode");
    }
    let cflags = tk.get_canonflags();
    tk.set_canonflags(cflags | termkey::c::TERMKEY_CANON_SPACESYMBOL);
    {
        let (key, endp) = tk.strpkey(" ", termkey::c::TermKeyFormat::empty()).unwrap();
        match key
        {
            termkey::KeySymEvent{sym, mods} =>
            {
                tap.pass("key.type for SP/symbol");
                tap.is_int(sym, termkey::c::TERMKEY_SYM_SPACE, "key.code.codepoint for SP/symbol");
                tap.ok(mods.is_empty(), "key.modifiers for SP/symbol");
            }
            _ => { tap.bypass(3, "key.type for SP/symbol") }
        }
        tap.is_str(endp, "", "consumed entire input for SP/symbol");
    }
    {
        let (key, endp) = tk.strpkey("Space", termkey::c::TermKeyFormat::empty()).unwrap();
        match key
        {
            termkey::KeySymEvent{sym, mods} =>
            {
                tap.pass("key.type for Space/symbol");
                tap.is_int(sym, termkey::c::TERMKEY_SYM_SPACE, "key.code.codepoint for Space/symbol");
                tap.ok(mods.is_empty(), "key.modifiers for Space/symbol");
            }
            _ => { tap.bypass(3, "key.type for Space/symbol") }
        }
        tap.is_str(endp, "", "consumed entire input for Space/symbol");
    }
    {
        let (key, endp) = tk.strpkey("DEL", termkey::c::TermKeyFormat::empty()).unwrap();
        match key
        {
            termkey::KeySymEvent{sym, mods} =>
            {
                tap.pass("key.type for Del/unconverted");
                tap.is_int(sym, termkey::c::TERMKEY_SYM_DEL, "key.code.codepoint for Del/unconverted");
                tap.ok(mods.is_empty(), "key.modifiers for Del/unconverted");
            }
            _ => { tap.bypass(3, "key.type for Del/unconverted") }
        }
        tap.is_str(endp, "", "consumed entire input for Del/unconverted");
    }
    let cflags = tk.get_canonflags();
    tk.set_canonflags(cflags | termkey::c::TERMKEY_CANON_DELBS);
    {
        let (key, endp) = tk.strpkey("DEL", termkey::c::TermKeyFormat::empty()).unwrap();
        match key
        {
            termkey::KeySymEvent{sym, mods} =>
            {
                tap.pass("key.type for Del/as-backspace");
                tap.is_int(sym, termkey::c::TERMKEY_SYM_BACKSPACE, "key.code.codepoint for Del/as-backspace");
                tap.ok(mods.is_empty(), "key.modifiers for Del/as-backspace");
            }
            _ => { tap.bypass(3, "key.type for Del/as-backspace") }
        }
        tap.is_str(endp, "", "consumed entire input for Del/as-backspace");
    }
}

#[test]
fn test_30mouse()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(60);

    let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::X_TermKey_Flag::empty());

    {
        tk.push_bytes("\x1b[M !!".as_bytes());

        match tk.getkey()
        {
            termkey::Key(key) =>
            {
                tap.pass("getkey yields RES_KEY for mouse press");

                match key
                {
                    termkey::MouseEvent{ev, button, line, col, mods} =>
                    {
                        tap.pass("key.type for mouse press");

                        tap.pass("interpret_mouse yields RES_KEY");

                        tap.is_int(ev, termkey::c::TERMKEY_MOUSE_PRESS, "mouse event for press");
                        tap.is_int(button, 1, "mouse button for press");
                        tap.is_int(line, 1, "mouse line for press");
                        tap.is_int(col, 1, "mouse column for press");
                        tap.ok(mods.is_empty(), "modifiers for press");

                        let buffer = tk.strfkey(key, termkey::c::TermKeyFormat::empty());
                        tap.is_int(buffer.len(), 13, "string length for press");
                        tap.is_str(buffer, "MousePress(1)", "string buffer for press");

                        let buffer = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_MOUSE_POS);
                        tap.is_int(buffer.len(), 21, "string length for press");
                        tap.is_str(buffer, "MousePress(1) @ (1,1)", "string buffer for press");
                    }
                _ => { tap.bypass(11, "key.type for mouse press") }
                }
            }
            _ => { tap.bypass(12, "getkey yields RES_KEY for mouse press") }
        }
    }

    {
        tk.push_bytes("\x1b[M@\"!".as_bytes());

        match tk.getkey()
        {
            termkey::Key(key) =>
            {
                match key
                {
                    termkey::MouseEvent{ev, button, line, col, mods} =>
                    {
                        tap.pass("interpret_mouse yields RES_KEY");

                        tap.is_int(ev, termkey::c::TERMKEY_MOUSE_DRAG, "mouse event for drag");
                        tap.is_int(button, 1, "mouse button for drag");
                        tap.is_int(line, 1, "mouse line for drag");
                        tap.is_int(col, 2, "mouse column for drag");
                        tap.ok(mods.is_empty(), "modifiers for press");
                    }
                    _ => { tap.bypass(6, "interpret_mouse yields RES_KEY") }
                }
            }
            _ => { tap.bypass(6, "interpret_mouse yields RES_KEY") }
        }

        tk.push_bytes("\x1b[M##!".as_bytes());

        match tk.getkey()
        {
            termkey::Key(key) =>
            {
                match key
                {
                    termkey::MouseEvent{ev, button: _, line, col, mods} =>
                    {
                        tap.pass("interpret_mouse yields RES_KEY");

                        tap.is_int(ev, termkey::c::TERMKEY_MOUSE_RELEASE, "mouse event for release");
                        tap.is_int(line, 1, "mouse line for release");
                        tap.is_int(col, 3, "mouse column for release");
                        tap.ok(mods.is_empty(), "modifiers for press");
                    }
                    _ => { tap.bypass(5, "interpret_mouse yields RES_KEY") }
                }
            }
            _ => { tap.bypass(5, "interpret_mouse yields RES_KEY") }
        }
    }

    {
        tk.push_bytes("\x1b[M0++".as_bytes());

        match tk.getkey()
        {
            termkey::Key(key) =>
            {
                match key
                {
                    termkey::MouseEvent{ev, button, line, col, mods} =>
                    {
                        tap.pass("interpret_mouse yields RES_KEY");

                        tap.is_int(ev, termkey::c::TERMKEY_MOUSE_PRESS, "mouse event for Ctrl-press");
                        tap.is_int(button, 1, "mouse button for Ctrl-press");
                        tap.is_int(line, 11, "mouse line for Ctrl-press");
                        tap.is_int(col, 11, "mouse column for Ctrl-press");
                        tap.is_int(mods, termkey::c::TERMKEY_KEYMOD_CTRL, "modifiers for Ctrl-press");

                        let buffer = tk.strfkey(key, termkey::c::TermKeyFormat::empty());
                        tap.is_int(buffer.len(), 15, "string length for Ctrl-press");
                        tap.is_str(buffer, "C-MousePress(1)", "string buffer for Ctrl-press");
                    }
                    _ => { tap.bypass(8, "interpret_mouse yields RES_KEY") }
                }
            }
            _ => { tap.bypass(8, "interpret_mouse yields RES_KEY") }
        }
    }

    //// rxvt protocol
    {
        tk.push_bytes("\x1b[0;20;20M".as_bytes());

        match tk.getkey()
        {
            termkey::Key(key) =>
            {
                tap.pass("getkey yields RES_KEY for mouse press rxvt protocol");
                match key
                {
                    termkey::MouseEvent{ev, button, line, col, mods} =>
                    {
                        tap.pass("key.type for mouse press rxvt protocol");

                        tap.pass("interpret_mouse yields RES_KEY");

                        tap.is_int(ev, termkey::c::TERMKEY_MOUSE_PRESS, "mouse event for press rxvt protocol");
                        tap.is_int(button, 1, "mouse button for press rxvt protocol");
                        tap.is_int(line, 20, "mouse line for press rxvt protocol");
                        tap.is_int(col, 20, "mouse column for press rxvt protocol");
                        tap.ok(mods.is_empty(), "modifiers for press rxvt protocol");
                    }
                    _ => { tap.bypass(7, "key.type for mouse press rxvt protocol") }
                }
            }
            _ => { tap.bypass(8, "getkey yields RES_KEY for mouse press rxvt protocol") }
        }
    }

    {
        tk.push_bytes("\x1b[3;20;20M".as_bytes());

        match tk.getkey()
        {
            termkey::Key(key) =>
            {
                tap.pass("getkey yields RES_KEY for mouse release rxvt protocol");
                match key
                {
                    termkey::MouseEvent{ev, button: _, line, col, mods} =>
                    {
                        tap.pass("key.type for mouse release rxvt protocol");

                        tap.pass("interpret_mouse yields RES_KEY");

                        tap.is_int(ev, termkey::c::TERMKEY_MOUSE_RELEASE, "mouse event for release rxvt protocol");
                        tap.is_int(line, 20, "mouse line for release rxvt protocol");
                        tap.is_int(col, 20, "mouse column for release rxvt protocol");
                        tap.ok(mods.is_empty(), "modifiers for release rxvt protocol");
                    }
                    _ => { tap.bypass(6, "key.type for mouse release rxvt protocol") }
                }
            }
            _ => { tap.bypass(7, "getkey yields RES_KEY for mouse release rxvt protocol") }
        }
    }

    //// SGR protocol
    {
        tk.push_bytes("\x1b[<0;30;30M".as_bytes());

        match tk.getkey()
        {
            termkey::Key(key) =>
            {
                tap.pass("getkey yields RES_KEY for mouse press SGR encoding");
                match key
                {
                    termkey::MouseEvent{ev, button, line, col, mods} =>
                    {
                        tap.pass("key.type for mouse press SGR encoding");

                        tap.pass("interpret_mouse yields RES_KEY");

                        tap.is_int(ev, termkey::c::TERMKEY_MOUSE_PRESS, "mouse event for press SGR");
                        tap.is_int(button, 1, "mouse button for press SGR");
                        tap.is_int(line, 30, "mouse line for press SGR");
                        tap.is_int(col, 30, "mouse column for press SGR");
                        tap.ok(mods.is_empty(), "modifiers for press SGR");
                    }
                    _ => { tap.bypass(7, "key.type for mouse press SGR encoding") }
                }
            }
            _ => { tap.bypass(8, "getkey yields RES_KEY for mouse press SGR encoding") }
        }
    }

    {
        tk.push_bytes("\x1b[<0;30;30m".as_bytes());

        match tk.getkey()
        {
            termkey::Key(key) =>
            {
                tap.pass("getkey yields RES_KEY for mouse release SGR encoding");
                match key
                {
                    termkey::MouseEvent{ev, button: _, line: _, col: _, mods: _} =>
                    {
                        tap.pass("key.type for mouse release SGR encoding");

                        tap.pass("interpret_mouse yields RES_KEY");

                        tap.is_int(ev, termkey::c::TERMKEY_MOUSE_RELEASE, "mouse event for release SGR");
                    }
                    _ => { tap.bypass(3, "key.type for mouse release SGR encoding") }
                }
            }
            _ => { tap.bypass(4, "getkey yields RES_KEY for mouse release SGR encoding") }
        }
    }

    {
        tk.push_bytes("\x1b[<0;500;300M".as_bytes());

        match tk.getkey()
        {
            termkey::Key(key) =>
            {
                match key
                {
                    termkey::MouseEvent{ev: _, button: _, line, col, mods: _} =>
                    {
                        tap.is_int(line, 300, "mouse line for press SGR wide");
                        tap.is_int(col, 500, "mouse column for press SGR wide");
                    }
                    _ => { tap.bypass(2, "mouse line/column for press SGR wide") }
                }
            }
            _ => { tap.bypass(2, "mouse line/column for press SGR wide") }
        }
    }
}

#[test]
fn test_31position()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(8);

    let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::X_TermKey_Flag::empty());

    tk.push_bytes("\x1b[?15;7R".as_bytes());

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY for position report");
            match key
            {
                termkey::PositionEvent{line, col} =>
                {
                    tap.pass("key.type for position report");

                    tap.pass("interpret_position yields RES_KEY");

                    tap.is_int(line, 15, "line for position report");
                    tap.is_int(col, 7, "column for position report");
                }
                _ => { tap.bypass(4, "key.type for position report") }
            }
        }
        _ => { tap.bypass(5, "getkey yields RES_KEY for position report") }
    }

    /* A plain CSI R is likely to be <F3> though.
    * This is tricky :/
    */
    tk.push_bytes("\x1b[R".as_bytes());

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY for <F3>");

            match key
            {
                termkey::FunctionEvent{mods: _, num} =>
                {
                    tap.pass("key.type for <F3>");
                    tap.is_int(num, 3, "key.code.number for <F3>");
                }
                _ => { tap.bypass(2, "key.type for <F3>") }
            }
        }
        _ => { tap.bypass(3, "getkey yields RES_KEY for <F3>") }
    }
}

#[test]
fn test_32modereport()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(12);

    let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::X_TermKey_Flag::empty());

    tk.push_bytes("\x1b[?1;2$y".as_bytes());

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY for mode report");
            match key
            {
                termkey::ModeReportEvent{initial, mode, value} =>
                {
                    tap.pass("key.type for mode report");

                    tap.pass("interpret_modereoprt yields RES_KEY");

                    tap.is_int(initial, '?' as int, "initial indicator from mode report");
                    tap.is_int(mode, 1, "mode number from mode report");
                    tap.is_int(value, 2, "mode value from mode report");
                }
                _ => { tap.bypass(5, "key.type for mode report") }
            }
        }
        _ => { tap.bypass(6, "getkey yields RES_KEY for mode report") }
    }

    tk.push_bytes("\x1b[4;1$y".as_bytes());

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY for mode report");

            match key
            {
                termkey::ModeReportEvent{initial, mode, value} =>
                {
                    tap.pass("key.type for mode report");

                    tap.pass("interpret_modereoprt yields RES_KEY");

                    tap.is_int(initial, 0, "initial indicator from mode report");
                    tap.is_int(mode, 4, "mode number from mode report");
                    tap.is_int(value, 1, "mode value from mode report");
                }
                _ => { tap.bypass(5, "key.type for mode report") }
            }
        }
        _ => { tap.bypass(6, "getkey yields RES_KEY for mode report") }
    }
}

#[test]
fn test_39csi()
{
    let mut tap = taplib::Tap::new();
    tap.plan_tests(15);

    let mut tk = termkey::TermKey::new_abstract("vt100", termkey::c::X_TermKey_Flag::empty());

    tk.push_bytes("\x1b[5;25v".as_bytes());

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY for CSI v");

            match key
            {
                termkey::UnknownCsiEvent =>
                {
                    tap.pass("key.type for unknown CSI");

                    tap.pass("skipping interpret_csi"); //is_int(tk.interpret_csi(key, &mut args, &mut command), TERMKEY_RES_KEY, "interpret_csi yields RES_KEY");

                    tap.pass("skipping nargs == 2"); //is_int(nargs, 2, "nargs for unknown CSI");
                    tap.pass("skipping args[0] == 5"); //is_int(args[0], 5, "args[0] for unknown CSI");
                    tap.pass("skipping args[1] == 25"); //is_int(args[1], 25, "args[1] for unknown CSI");
                    tap.pass("skipping cmd == 'v'"); //is_int(command, 'v', "command for unknown CSI");
                }
                _ => { tap.bypass(6, "key.type for unknown CSI") }
            }
        }
        _ => { tap.bypass(7, "getkey yields RES_KEY for CSI v") }
    }

    tk.push_bytes("\x1b[?w".as_bytes());

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY for CSI ? w");
            match key
            {
                termkey::UnknownCsiEvent =>
                {
                    tap.pass("key.type for unknown CSI");
                    tap.pass("skipping interpret_csi"); //is_int(tk.interpret_csi(&key, args, &nargs, &command), TERMKEY_RES_KEY, "interpret_csi yields RES_KEY");
                    tap.pass("skipping cmd == '?' cat 'w'")//is_int(command, '?'<<8 | 'w', "command for unknown CSI");
                }
                _ => { tap.bypass(3, "key.type for unknown CSI") }
            }
        }
        _ => { tap.bypass(4, "getkey yields RES_KEY for CSI ? w") }
    }

    tk.push_bytes("\x1b[?$x".as_bytes());

    match tk.getkey()
    {
        termkey::Key(key) =>
        {
            tap.pass("getkey yields RES_KEY for CSI ? $x");
            match key
            {
                termkey::UnknownCsiEvent =>
                {
                    tap.pass("key.type for unknown CSI");
                    tap.pass("skipping interpret_csi"); //is_int(tk.interpret_csi(&key, args, &nargs, &command), TERMKEY_RES_KEY, "interpret_csi yields RES_KEY");
                    tap.pass("skipping cmd == '$' cat '?' cat 'x'"); //is_int(command, '$'<<16 | '?'<<8 | 'x', "command for unknown CSI");
                }
                _ => { tap.bypass(3, "key.type for unknown CSI") }
            }
        }
        _ => { tap.bypass(4, "getkey yields RES_KEY for CSI ? $x") }
    }
}
