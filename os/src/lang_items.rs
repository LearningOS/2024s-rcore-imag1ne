use core::panic::PanicInfo;
use crate::{print, println};
use crate::sbi::shutdown;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("[kernel] Panicked");

    match info.location() {
        None => println!(": {}", info.message().unwrap()),
        Some(location) => {
            println!(" at {}:{} {}", location.file(), location.line(), info.message().unwrap());
        }
    }

    shutdown();

    loop {}
}
