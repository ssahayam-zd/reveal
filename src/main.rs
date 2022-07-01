use walkdir::{DirEntry, WalkDir};
use std::io::Write;
use std::path::{PathBuf, Path};
use std::process::Command;
// use std::str::from_utf8;
use std::fs;

fn main() {

  let working_dir = "/Users/sanjiv.sahayam/ziptemp/tmp-proto/7.273.0-4dd7dac3-SNAPSHOT";
  let target_dir = "/Users/sanjiv.sahayam/ziptemp/tmp-proto/7.273.0-4dd7dac3-SNAPSHOT-output";

  WalkDir::new(working_dir)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(is_valid_file)
    .for_each(|entry|{
      let p = entry.path();
      let class_name = p.file_name().unwrap().to_string_lossy().replace(".class", "");
      let parent_path = p.parent().unwrap().to_string_lossy();
      let (_, relative_dir) = parent_path.split_once(working_dir).unwrap();    
      let relative_parent_path = relative_dir.strip_prefix("/").unwrap_or(relative_dir);
      let parent_dotted_path = relative_parent_path.replace("/", ".");
      decompile_class(
        ParentDottedPath(parent_dotted_path),
        ParentRelativePath(relative_parent_path.to_owned()),
        ClassName(class_name),
        WorkingDir(Path::new(working_dir).to_owned()),
        TargetDir(Path::new(target_dir).to_owned())
      )  })
}

struct ParentDottedPath(String);
struct ParentRelativePath(String);
struct ClassName(String);
struct WorkingDir(PathBuf);
struct TargetDir(PathBuf);

fn decompile_class(parent_dotted_path: ParentDottedPath, relative_parent_path: ParentRelativePath, class_name: ClassName, working_dir: WorkingDir, target_dir: TargetDir) {
  let output_dir = target_dir.0.join(relative_parent_path.0);
  let dotted_scala_file = format!("{}.{}", parent_dotted_path.0, class_name.0);
  let target_scala_file = format!("{}/{}.scala", output_dir.clone().to_string_lossy(), class_name.0);

  if !output_dir.is_dir() {
    fs::create_dir_all(output_dir.clone()).unwrap()
  }

  // println!("scalap {} > {}", dotted_scala_file, target_scala_file);
  // println!("###> {}", output_dir.clone().to_string_lossy());
  println!("writing {}", dotted_scala_file);

  let output = 
    Command::new("scalap")
    .current_dir(working_dir.0)
    .arg(dotted_scala_file)
    .output()
    .unwrap();

    let mut output_file = fs::File::create(target_scala_file).unwrap();
    output_file.write_all(&output.stdout).unwrap();

    println!("{}", output.status)
}

fn is_valid_file(entry: &DirEntry) -> bool {
  let is_dir = entry.file_type().is_dir();

  let is_nested_class = 
    if entry.file_type().is_file() {
      let file_name = entry.path().file_name().unwrap().to_string_lossy();
      match file_name.rfind("$") {
        Some(_) => true, // found a $, so we'll skip this
        None => false
      }
    } else {
      true
    };

    !(is_dir || is_nested_class)
}
