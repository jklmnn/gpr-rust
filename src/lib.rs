use std::{
    fmt,
    path::{Path, PathBuf},
};

#[macro_use]
extern crate enum_display_derive;

mod binding;
mod error;

pub use self::binding::{finalize, initialize};

#[derive(Debug)]
pub struct Project {
    file: PathBuf,
    tree: binding::Tree,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LibraryKind {
    Static,
    Dynamic,
}

impl fmt::Display for LibraryKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryKind::Static => write!(f, "static"),
            LibraryKind::Dynamic => write!(f, "dylib"),
        }
    }
}

macro_rules! single {
    ($self: expr, $name: expr) => {
        $self.get_single_attribute_value($name, $self.tree.get_attribute($name)?.value)
    };
}

macro_rules! list {
    ($self: expr, $name: expr) => {
        $self.get_list_attribute_value($name, $self.tree.get_attribute($name)?.value)
    };
}

impl Project {
    pub fn load(file: &Path) -> Result<Project, error::Error> {
        let tree = binding::Tree::load(file)?;
        Ok(Project {
            file: file.canonicalize()?,
            tree,
        })
    }

    fn basepath(&self) -> PathBuf {
        self.file.as_path().parent().unwrap().to_path_buf()
    }

    fn get_single_attribute_value(
        &self,
        name: &str,
        attr: binding::AttributeValue,
    ) -> Result<String, error::Error> {
        if let binding::AttributeValue::Single(result) = attr {
            Ok(result)
        } else {
            Err(error::Error::invalid_attribute_value(
                &self.file, name, &attr,
            ))
        }
    }

    fn get_list_attribute_value(
        &self,
        name: &str,
        attr: binding::AttributeValue,
    ) -> Result<Vec<String>, error::Error> {
        if let binding::AttributeValue::List(result) = attr {
            Ok(result)
        } else {
            Err(error::Error::invalid_attribute_value(
                &self.file, name, &attr,
            ))
        }
    }

    pub fn name(&self) -> Result<String, error::Error> {
        single!(self, "name")
    }

    pub fn library_name(&self) -> Result<String, error::Error> {
        single!(self, "library_name")
    }

    pub fn library_dir(&self) -> Result<PathBuf, error::Error> {
        Ok(self
            .basepath()
            .as_path()
            .join(single!(self, "library_dir")?))
    }

    pub fn library_kind(&self) -> Result<LibraryKind, error::Error> {
        match single!(self, "library_kind")?.as_str() {
            "static" | "static-pic" => Ok(LibraryKind::Static),
            "dynamic" | "relocatable" => Ok(LibraryKind::Dynamic),
            value => Err(error::Error::invalid_attribute(
                &self.file,
                "library_kind",
                value,
            )),
        }
    }

    pub fn source_dirs(&self) -> Result<Vec<String>, error::Error> {
        list!(self, "source_dirs")
    }

    pub fn gprbuild_args(&self) -> Result<Vec<String>, error::Error> {
        Ok(vec![
            String::from("-P"),
            String::from(self.file.to_str().ok_or_else(|| {
                error::Error::from_code(
                    error::Code::UnknownError,
                    "InvalidFile",
                    "Failed to get path to project file",
                )
            })?),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{os::raw::c_int, process::Command};
    extern crate libloading as lib;

    macro_rules! prj {
        ($file: expr) => {
            Project::load(Path::new($file)).unwrap()
        };
    }

    #[test]
    fn test_name() {
        initialize();
        let prj = prj!("testdata/testlib.gpr");
        assert_eq!(prj.name().unwrap(), "testlib");
    }

    #[test]
    fn test_library_name() {
        initialize();
        let prj = prj!("testdata/testlib.gpr");
        assert_eq!(prj.library_name().unwrap(), "test");
    }

    #[test]
    fn test_library_dir() {
        initialize();
        let prj = prj!("testdata/testlib.gpr");
        assert_eq!(
            prj.library_dir().unwrap(),
            Path::new("testdata").canonicalize().unwrap().join("lib")
        );
    }

    #[test]
    fn test_library_kind() {
        initialize();
        let prj = prj!("testdata/testlib.gpr");
        assert_eq!(prj.library_kind().unwrap(), LibraryKind::Static);
    }

    #[test]
    fn test_source_dirs() {
        initialize();
        let prj = prj!("testdata/testlib.gpr");
        assert_eq!(prj.source_dirs().unwrap(), vec!["src", "src2"]);
    }

    #[test]
    fn test_build() {
        initialize();
        let prj = prj!("testdata/test2.gpr");
        assert_eq!(prj.name().unwrap(), "test2");
        assert_eq!(prj.library_name().unwrap(), "test2");
        assert_eq!(prj.library_kind().unwrap(), LibraryKind::Dynamic);
        assert_eq!(
            prj.library_dir().unwrap(),
            Path::new("testdata").canonicalize().unwrap().join("lib")
        );
        Command::new("gprbuild")
            .args(prj.gprbuild_args().unwrap())
            .args(["-p", "-f"])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        let test2 = lib::Library::new(format!(
            "{}/lib{}.so",
            prj.library_dir().unwrap().to_str().unwrap(),
            prj.library_name().unwrap()
        ))
        .unwrap();
        unsafe {
            let test2init: lib::Symbol<unsafe extern "C" fn()> = test2.get(b"test2init").unwrap();
            let test2final: lib::Symbol<unsafe extern "C" fn()> = test2.get(b"test2final").unwrap();
            let test2_add: lib::Symbol<unsafe extern "C" fn(c_int, c_int) -> c_int> =
                test2.get(b"test2_add").unwrap();
            test2init();
            assert_eq!(test2_add(42, 24), 66);
            test2final();
        }
    }

    #[test]
    fn test_invalid_project() {
        initialize();
        assert!(Project::load(Path::new("testdata/invalid.gpr")).is_err());
    }

    #[test]
    fn test_library_kind_display() {
        assert_eq!(format!("{}", LibraryKind::Dynamic), "dylib");
        assert_eq!(format!("{}", LibraryKind::Static), "static");
    }
}
