use std::path::{Path, PathBuf};

#[macro_use]
extern crate enum_display_derive;

mod binding;
mod error;

pub use self::binding::{initialize, finalize};

pub struct Project {
    basedir: PathBuf,
    tree: binding::Tree,
}

impl Project {
    pub fn load(file: &Path) -> Result<Project, error::Error> {
        let tree = binding::Tree::load(file)?;
        let basepath = file.canonicalize()?.parent().unwrap().to_path_buf();
        Ok(Project {
            basedir: basepath,
            tree: tree,
        })
    }

    pub fn name(&self) -> Result<String, error::Error> {
        Ok(self.tree.get_attribute("name")?.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        binding::initialize();
        let prj = Project::load(Path::new("testdata/testlib.gpr")).unwrap();
        assert_eq!(prj.name().unwrap(), "testlib");
    }
}
