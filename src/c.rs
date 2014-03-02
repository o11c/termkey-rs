use std::libc::c_int;

// Better to handle in makefile
//#[link(name = "termkey")]
extern
{
    pub fn termkey_check_version(major: c_int, minor: c_int);
}
