use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
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

impl Project {
    pub fn load(file: &Path) -> Result<Project, error::Error> {
        let tree = binding::Tree::load(file)?;
        Ok(Project {
            file: file.canonicalize()?.to_path_buf(),
            tree: tree,
        })
    }

    fn basepath(&self) -> PathBuf {
        self.file.as_path().parent().unwrap().to_path_buf()
    }

    pub fn name(&self) -> Result<String, error::Error> {
        Ok(self.tree.get_attribute("name")?.value)
    }

    pub fn library_name(&self) -> Result<String, error::Error> {
        Ok(self.tree.get_attribute("library_name")?.value)
    }

    pub fn library_dir(&self) -> Result<PathBuf, error::Error> {
        Ok(self
            .basepath()
            .as_path()
            .join(self.tree.get_attribute("library_dir")?.value))
    }

    pub fn library_kind(&self) -> Result<LibraryKind, error::Error> {
        match self.tree.get_attribute("library_kind")?.value.as_str() {
            "static" | "static-pic" => Ok(LibraryKind::Static),
            "dynamic" | "relocatable" => Ok(LibraryKind::Dynamic),
            value => Err(error::Error::invalid_attribute(
                &self.file,
                "library_kind",
                value,
            )),
        }
    }

    pub fn build<I, S>(&self, args: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new("gprbuild")
            .arg("-P")
            .arg(self.file.to_str().unwrap())
            .args(args)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        prj.build(["-p", "-f"]);
    }
}
