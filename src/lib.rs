#[crate_id = "termkey#0.17.0"];
#[crate_type = "dylib"];

mod c;

pub fn hello()
{
    unsafe
    {
        c::termkey_check_version(0, 17);
    }
    println!("Hello from libtermkey");
}
