use std::path::{Path, PathBuf};

pub struct ParentDottedPath(String);

impl ParentDottedPath {
  pub fn value(&self) -> String {
    self.0.clone()
  } 

  pub fn new(package_str: &str) -> Self {
    Self(package_str.to_owned())
  }

}


pub struct ParentRelativePath(PathBuf);

impl ParentRelativePath {
  pub fn value(&self) -> String {
    self.0.to_string_lossy().to_string()
  }  

  pub fn new<P: AsRef<Path>>(relative_path: P) -> Self {
    Self(relative_path.as_ref().to_path_buf())
  }
}


pub struct ClassName(String);

impl ClassName {
  pub fn value(&self) -> String {
    self.0.clone()
  }

  pub fn new(class_name: &str) -> Self {
    Self(class_name.to_owned())
  }    
}

#[derive(Clone)]
pub struct WorkingDir(PathBuf);

impl AsRef<Path> for WorkingDir {
  fn as_ref(&self) -> &Path {
    self.0.as_ref()
  }
}

impl WorkingDir {
  pub fn to_string_lossy(&self) -> String {
    self.0.to_string_lossy().to_string()
  }

  pub fn new(path: &str) -> Self {
    Self(Path::new(path).to_path_buf())
  }  
}

#[derive(Clone)]
pub struct TargetDir(PathBuf);


impl AsRef<Path> for TargetDir {
  fn as_ref(&self) -> &Path {
    self.0.as_ref()
  }
}

impl TargetDir {

  pub fn new(path: &str) -> Self {
    Self(Path::new(path).to_path_buf())
  }

  pub fn to_string_lossy(&self) -> String {
    self.0.to_string_lossy().to_string()
  }

  pub fn join<P: AsRef<Path>>(&self, path: P) -> Self {
    Self(self.0.join(path))
  }

  pub fn is_dir(&self) -> bool {
    self.0.is_dir()
  }
}

pub struct ScalapArguments {
  pub parent_dotted_path: ParentDottedPath,
  pub parent_relative_path: ParentRelativePath,
  pub class_name: ClassName,
  pub working_dir: WorkingDir,
  pub target_dir: TargetDir
}


