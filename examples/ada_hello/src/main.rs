extern "C" {
    fn ada_hello(a: i32, b: i32) -> i32;
}

fn main() {
    println!("Hello from Rust!");
    unsafe {
        let m: i32 = ada_hello(7, 4);
        println!("There is m calculated in Ada: {}", m);
    }
}

// The rust build process should fail when Ada fail
// Assumptions of the Rust program for Ada
// 1. It is going to be an i32 integer --> Integer on the Ada side
// We don't know the size of the Integer
// usize and isize (used to be called int) =?= Integer in ada
// "The size of this primitive is how many bytes it takes to reference any location in memory."
// --> they are not meant for calculation
// calculation => use sized type for calculation
// semantically = Integer_ada are machine dependent numbers// Rust they are not machine dependent
// always use specific type sizes
// try that with 128

// C's is using machine-dependent types
// unsize == "we cannot ask the compiler what the size is"
// that was a huge problem before, Ada has 32 bits integers!

// GNAT says 32 bit Integer are signed
// https://gcc.gnu.org/onlinedocs/gcc-3.2.3/gnat_rm/Implementation-Defined-Characteristics.html #13
// ada specs: https://ada-lang.io/docs/arm/AA-3/AA-3.5/#354--integer-types

// how do we work with constrains in Rust -- Ada?
// weak point of rust // strong point of Ada how to get creative

// I can take the stance that I expect that the gnat compiler will be used. Relying on the tools?
// the standards?

// ask Yannick about how many compilers, and should we look at other compilers?
// In rust --> no standards, no docs to write the intention

// there is nothing outside the ferrocene spec

// constrained integer or modular integers? make recommendations to rust

// ada enums to Rust

// ABI is just a contract --> explains why the function was unsafe!

// FFI is not a standard
// ABi is mandated by the processor vendor, the facto standard

// does Ada has a signed Integer
// is there Ada libraries out there that gives you scope sized

// write some tests

// rust is nice because it is upfront with its type so no long_long long_long_short_long
