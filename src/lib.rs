use std::{
    os::raw::{c_int, c_char},
    ffi::CString,
    collections::HashMap,
    path::Path,
    ptr::null_mut,
};
use serde::Deserialize;
use serde_json::json;

extern "C" {
    fn gpr2cinit();
    fn gpr2cfinal();
    fn gpr2_request(fun: c_int, request: *const c_char, answer: *mut *mut c_char) -> c_int;
    fn gpr2_free_answer(answer: *const c_char);
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Result {
    Tree {
        id: String,
        root_view: String,
        config_view: Option<String>,
        runtime_view: Option<String>,
        target: String,
        canonical_target: String,
        search_paths: Vec<String>,
        src_subdirs: Option<String>,
        subdirs: Option<String>,
        build_path: Option<String>,
        views: Vec<String>,
        context: HashMap<String, String>
    }
}

#[derive(Deserialize)]
struct Answer {
    result: Result,
    status: i32,
    error_msg: String,
    error_name: String
}

impl crate::Result {
    fn load(file: &Path) -> () {//Option<Tree> {
        let request = json!({
            "filename": file.to_str().unwrap()
        }).to_string();
        let mut answer: *mut c_char = null_mut();
        let request = CString::new(request).unwrap();
        let answer_string: String;
        let result: c_int;
        unsafe {
            result = gpr2_request(1, request.as_ptr(), &mut answer);
            answer_string = match CString::from_raw(answer).into_string() {
                Ok(s) => s,
                Err(e) => panic!("blubb: {}", e),
            };
        }
        println!("{}: {}", result, answer_string);
        let answer: Answer = serde_json::from_str(&answer_string).expect("invalid json");
        //serde_json::from_str::<HashMap<&str, Tree>>(&answer_string).expect("invalid json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        unsafe {
            gpr2cinit();
        }
    }

    #[test]
    fn test_load() {
        unsafe {
            gpr2cinit();
        }
        crate::Result::load(Path::new("testdata/testlib.gpr"));
    }

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
