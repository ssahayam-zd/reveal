use walkdir::{DirEntry, WalkDir};
use std::io::Write;
use std::path::{PathBuf, Path};
use std::process::Command;
use std::fs;

type R<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> R<()> {

  //TODO: Accept these params
  let working_dir = "/Users/sanjiv.sahayam/ziptemp/tmp-proto/7.273.0-4dd7dac3-SNAPSHOT";
  let target_dir = "/Users/sanjiv.sahayam/ziptemp/tmp-proto/7.273.0-4dd7dac3-SNAPSHOT-output2";

  walk_tree(
  WorkingDir(Path::new(working_dir).to_path_buf()), 
  TargetDir(Path::new(target_dir).to_path_buf())
  )
}

fn walk_tree(working_dir: WorkingDir, target_dir: TargetDir) -> R<()> {
  let results: Result<Vec<()>, Box<dyn std::error::Error>> = 
    WalkDir::new(working_dir.clone().0)
      .into_iter()
      .filter_map(|e| e.ok())
      .filter(is_valid_file)
      .map(|entry|{
        let p = entry.path();
        let class_name = p.file_name().ok_or(raise_error("Could not get file name"))?.to_string_lossy().replace(".class", "");
        let parent_path = p.parent().ok_or(raise_error("no parent dir"))?.to_string_lossy();
        let (_, relative_dir) = (*parent_path).split_once(working_dir.0.to_string_lossy().as_ref()).ok_or("can't detect relative dir")?;    
        let relative_parent_path = relative_dir.strip_prefix("/").unwrap_or(relative_dir);
        let parent_dotted_path = relative_parent_path.replace("/", ".");
        decompile_class(
          ParentDottedPath(parent_dotted_path),
          ParentRelativePath(relative_parent_path.to_owned()),
          ClassName(class_name),
          working_dir.clone(),
          target_dir.clone()
        )
      }).collect();


  results.map(|_| ())
}

fn raise_error(message: &str) -> Box::<dyn std::error::Error> {
  Box::<dyn std::error::Error>::from(message)
}

struct ParentDottedPath(String);
struct ParentRelativePath(String);
struct ClassName(String);

#[derive(Clone)]
struct WorkingDir(PathBuf);

#[derive(Clone)]
struct TargetDir(PathBuf);

fn decompile_class(parent_dotted_path: ParentDottedPath, relative_parent_path: ParentRelativePath, class_name: ClassName, working_dir: WorkingDir, target_dir: TargetDir) -> R<()> {
  let output_dir = target_dir.0.join(relative_parent_path.0);
  let dotted_scala_file = format!("{}.{}", parent_dotted_path.0, class_name.0);
  let target_scala_file = format!("{}/{}.scala", output_dir.clone().to_string_lossy(), class_name.0);

  if !output_dir.is_dir() {
    fs::create_dir_all(output_dir.clone())?
  }

  // println!("scalap {} > {}", dotted_scala_file, target_scala_file);
  // println!("###> {}", output_dir.clone().to_string_lossy());
  println!("writing {}", dotted_scala_file);

  let output = 
    Command::new("scalap")
    .current_dir(working_dir.0)
    .arg(dotted_scala_file)
    .output()?;

    let mut output_file = fs::File::create(target_scala_file)?;
    output_file.write_all(&output.stdout)?;

    println!("{}", output.status);

    Ok(())
}

fn is_valid_file(entry: &DirEntry) -> bool {
  let is_dir = entry.file_type().is_dir();

  let is_nested_class = 
    if entry.file_type().is_file() {
      let file_name_has_dollar = 
        entry
          .path()
          .file_name()
          .map(|os_str| os_str.to_string_lossy())
          .and_then(|s| s.rfind("$"));
      
      file_name_has_dollar.map_or(false, |_| true)
    } else {
      false //not a nested class if it's not a file
    };

    !(is_dir || is_nested_class)
}
