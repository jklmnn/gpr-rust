extern "C" {
    fn ada_hello(a: i32, b: i32) -> i32;
}

fn main() {
    println!("Hello from Rust!");
    let x: i32;
    unsafe {
        x = ada_hello(5, 20);
    }
    println!("Result: {}", x);
}
