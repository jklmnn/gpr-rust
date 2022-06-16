use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    ffi::CString,
    os::raw::{c_char, c_int},
    path::Path,
    ptr::null_mut,
};

use super::error;

extern "C" {
    fn gpr2cinit();
    fn gpr2cfinal();
    fn gpr2_request(fun: c_int, request: *const c_char, answer: *mut *mut c_char) -> c_int;
    fn gpr2_free_answer(answer: *const c_char);
}

fn raw_request(fun_id: i32, request: &str) -> std::result::Result<String, error::Error> {
    let mut answer: *mut c_char = null_mut();
    let request = CString::new(request).unwrap();
    unsafe {
        let _ = gpr2_request(fun_id as c_int, request.as_ptr(), &mut answer);
        Ok(CString::from_raw(answer).into_string()?)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Result {
    Tree(Tree),
    Attribute(AttributeWrapper),
}

#[derive(Deserialize)]
pub struct Tree {
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
    context: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct Attribute {
    pub value: String,
    is_default: bool,
}

#[derive(Deserialize)]
pub struct AttributeWrapper {
    attribute: Attribute,
}

#[derive(Deserialize)]
struct Answer {
    result: Result,
    status: i32,
    error_msg: String,
    error_name: String,
}

pub fn initialize() {
    unsafe {
        gpr2cinit();
    }
}

pub fn finalize() {
    unsafe {
        gpr2cfinal();
    }
}

impl Tree {
    pub fn load(file: &Path) -> std::result::Result<Tree, error::Error> {
        let request = json!({
            "filename": file.to_str().unwrap()
        })
        .to_string();
        let raw_answer = raw_request(1, &request)?;
        let answer: Answer = serde_json::from_str(&raw_answer)?;
        match answer.status {
            0 => match answer.result {
                Result::Tree(t) => Ok(t),
                _ => Err(error::Error::from_code(
                    error::Code::UnknownError,
                    "InvalidResponse",
                    &raw_answer,
                )),
            },
            _ => {
                Err(
                    error::Error::from_status(answer.status, &answer.error_name, &answer.error_msg)
                        .unwrap(),
                )
            }
        }
    }

    pub fn get_attribute(&self, name: &str) -> std::result::Result<Attribute, error::Error> {
        let request = json!({
            "tree_id": self.id,
            "view_id": self.root_view,
            "name": name
        })
        .to_string();
        let raw_answer = raw_request(8, &request)?;
        let answer: Answer = serde_json::from_str(&raw_answer)?;
        match answer.status {
            0 => match answer.result {
                Result::Attribute(a) => Ok(a.attribute),
                _ => Err(error::Error::from_code(
                    error::Code::UnknownError,
                    "InvalidResponse",
                    &raw_answer,
                )),
            },
            _ => {
                Err(
                    error::Error::from_status(answer.status, &answer.error_name, &answer.error_msg)
                        .unwrap(),
                )
            }
        }
    }
}
