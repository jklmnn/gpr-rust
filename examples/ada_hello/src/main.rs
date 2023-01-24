extern "C" {
    fn ada_hello() -> i32;
}

fn main() {
    println!("Hello from Rust!");
    unsafe {
        let m: i32 = ada_hello();
        println!("Hello m: {}", m);
    }
}
