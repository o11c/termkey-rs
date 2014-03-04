extern crate native;

extern crate termkey;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main()
{
    let mouse = 0; // TODO parse arg -m, default 1000
    let mouse_proto = 0; // TODO parse arg -p (no default)
    let format = termkey::c::TERMKEY_FORMAT_VIM;

    let mut tk = termkey::TermKey::new(0, termkey::c::TERMKEY_FLAG_SPACESYMBOL|termkey::c::TERMKEY_FLAG_CTRLC);
    if !(tk.get_flags() & termkey::c::TERMKEY_FLAG_UTF8).is_empty()
    {
        println!("Termkey in UTF-8 mode")
    }
    if !(tk.get_flags() & termkey::c::TERMKEY_FLAG_RAW).is_empty()
    {
        println!("Termkey in RAW mode")
    }
    if mouse != 0
    {
        println!("\x1b[?{}hMouse mode active", mouse);
        if mouse_proto != 0
        {
            println!("\x1b[?{}h", mouse_proto);
        }
    }
    loop
    {
        match tk.waitkey()
        {
            termkey::Eof => break,
            termkey::Key(key) =>
            {
                match tk.strfkey(&key, format)
                {
                    Some(s) => { println!("Key {}", s) }
                    None => {}
                }
                match key
                {
                    termkey::MouseEvent{mods: _, ev: _, button: _, line, col} =>
                    {
                        println!("Mouse (printing unimplemented, sorry) at line={}, col={}\n", line, col)
                    }
                    termkey::PositionEvent{line, col} =>
                    {
                        println!("Cursor position report at line={}, col={}\n", line, col)
                    }
                    termkey::ModeReportEvent{initial, mode, value} =>
                    {
                        let initial_str = if initial != 0 { "DEC" } else { "ANSI" };
                        println!("Mode report {} mode {} = {}\n", initial_str, mode, value)
                    }
                    termkey::UnknownCsiEvent{..} =>
                    {
                        println!("Unrecognised CSI (printing unimplemented, sorry)\n")
                    }
                    _ => {}
                }
                match key
                {
                    termkey::UnicodeEvent{mods, codepoint, utf8: _} =>
                    {
                        if !(mods & termkey::c::TERMKEY_KEYMOD_CTRL).is_empty() && (codepoint == 'C' || codepoint == 'c')
                        {
                            break;
                        }
                        if mods.is_empty() && codepoint == '?'
                        {
                            // println!("\x1b[?6n"); // DECDSR 6 == request cursor position
                            println!("\x1b[?1$p"); // DECRQM == request mode, DEC origin mode
                        }
                    }
                    _ => {}
                }
            }
            termkey::Error =>
            {
                println!("Error of some sort")
            }
            _ => { fail!() }
        }
    }
    if mouse != 0
    {
        println!("\x1b[?{}lMouse mode deactivated", mouse)
    }
}
