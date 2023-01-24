extern "C" {
    fn ada_hello();
}

fn main() {
    println!("Hello from Rust!");
    unsafe {
        ada_hello();
    }
}
