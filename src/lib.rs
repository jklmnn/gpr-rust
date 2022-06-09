use std::{os::raw::{c_int, c_char}, ffi::CStr, str};

extern "C" {
    fn gpr2cinit();
    fn gpr2cfinal();
    fn gpr2_request(fun: c_int, request: *const c_char, answer: *mut *mut c_char) -> c_int;
    fn gpr2_free_answer(answer: *const c_char);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
