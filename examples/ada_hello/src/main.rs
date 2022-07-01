extern "C" {
    fn adahelloinit();
    fn adahellofinal();
    fn ada_hello();
}

fn main() {
    println!("Hello from Rust!");
    unsafe {
        adahelloinit();
        ada_hello();
        adahellofinal();
    }
}
