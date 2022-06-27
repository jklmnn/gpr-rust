use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    ffi::CString,
    fmt,
    os::raw::{c_char, c_int},
    path::Path,
    ptr::null_mut,
};

use super::error;

extern "C" {
    fn gpr2cinit();
    fn gpr2cfinal();
    fn gpr2_request(fun: c_int, request: *const c_char, answer: *mut *mut c_char) -> c_int;
    #[allow(dead_code)] //CString::from_raw frees the answer already
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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Result {
    Tree(Box<Tree>),
    Attribute(AttributeWrapper),
}

#[derive(Debug, Deserialize)]
pub struct Tree {
    id: String,
    root_view: String,
    #[allow(dead_code)]
    config_view: Option<String>,
    #[allow(dead_code)]
    runtime_view: Option<String>,
    #[allow(dead_code)]
    target: String,
    #[allow(dead_code)]
    canonical_target: String,
    #[allow(dead_code)]
    search_paths: Vec<String>,
    #[allow(dead_code)]
    src_subdirs: Option<String>,
    #[allow(dead_code)]
    subdirs: Option<String>,
    #[allow(dead_code)]
    build_path: Option<String>,
    #[allow(dead_code)]
    views: Vec<String>,
    #[allow(dead_code)]
    context: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    Single(String),
    List(Vec<String>),
}

impl fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributeValue::Single(_) => write!(f, "Single"),
            AttributeValue::List(_) => write!(f, "List"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Attribute {
    pub value: AttributeValue,
    #[allow(dead_code)]
    is_default: bool,
}

#[derive(Debug, Deserialize)]
struct AttributeWrapper {
    attribute: Attribute,
}

#[derive(Debug, Deserialize)]
struct EmptyResult {}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ResultWrapper {
    Valid(Result),
    Empty(EmptyResult),
}

#[derive(Deserialize)]
struct Answer {
    result: ResultWrapper,
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

fn unwrap_result(answer: Answer) -> std::result::Result<Result, error::Error> {
    if let ResultWrapper::Valid(result) = answer.result {
        Ok(result)
    } else {
        Err(
            error::Error::from_status(answer.status, &answer.error_name, &answer.error_msg)
                .unwrap(),
        )
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
        match unwrap_result(answer)? {
            Result::Tree(t) => Ok(*t),
            _ => Err(error::Error::from_code(
                error::Code::UnknownError,
                "InvalidResponse",
                &raw_answer,
            )),
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
        match unwrap_result(answer)? {
            Result::Attribute(a) => Ok(a.attribute),
            _ => Err(error::Error::from_code(
                error::Code::UnknownError,
                "InvalidResponse",
                &raw_answer,
            )),
        }
    }
}
