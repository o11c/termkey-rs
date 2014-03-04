extern crate termkey;

#[test]
fn test_basic()
{
    let mut term: termkey::TermKey = termkey::TermKey::new(0, termkey::c::X_TermKey_Flag::empty());
    term.stop();
    term.start();
}

