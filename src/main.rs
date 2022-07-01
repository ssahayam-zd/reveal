use walkdir::{DirEntry, WalkDir};
use std::io::Write;
use std::process::Command;
use std::fs;
use model::*;

mod model;

type R<T> = Result<T, Box<dyn std::error::Error>>;

const SUCCESS: &str = "success";
const FAILURE: &str = "failed";

fn main() -> R<()> {

  //TODO: Accept these params
  let working_dir = "/Users/sanjiv.sahayam/ziptemp/tmp-proto/7.273.0-4dd7dac3-SNAPSHOT";
  let target_dir = "/Users/sanjiv.sahayam/ziptemp/tmp-proto/7.273.0-4dd7dac3-SNAPSHOT-output2";

  walk_tree(
  WorkingDir::new(working_dir), 
  TargetDir::new(target_dir)
  )
}

fn walk_tree(working_dir: WorkingDir, target_dir: TargetDir) -> R<()> {
  let results: Result<Vec<()>, Box<dyn std::error::Error>> = 
    WalkDir::new(working_dir.clone())
      .into_iter()
      .filter_map(|e| e.ok())
      .filter(is_valid_file)
      .map(|entry|{
        let scalap_args = get_scalap_args(entry, &working_dir, &target_dir)?;     
        decompile_class(scalap_args)
      }).collect();


  results.map(|_| ())
}

fn get_scalap_args(entry: DirEntry, working_dir: &WorkingDir, target_dir: &TargetDir) -> R<ScalapArguments> {
    let p = entry.path();
    let class_name = p.file_name().ok_or(raise_error("Could not get file name"))?.to_string_lossy().replace(".class", "");
    let parent_path = p.parent().ok_or(raise_error("no parent dir"))?.to_string_lossy();
    let (_, relative_dir) = parent_path.split_once(working_dir.to_string_lossy().as_str()).ok_or("can't detect relative dir")?;    
    let relative_parent_path = relative_dir.strip_prefix("/").unwrap_or(relative_dir);
    let parent_dotted_path = relative_parent_path.replace("/", ".");

    let result =
      ScalapArguments {
        parent_dotted_path: ParentDottedPath::new(parent_dotted_path.as_ref()),
        parent_relative_path: ParentRelativePath::new(relative_parent_path),
        class_name: ClassName::new(class_name.as_ref()),
        working_dir: working_dir.clone(),
        target_dir: target_dir.clone()
      };

    Ok(result)
}

fn raise_error(message: &str) -> Box::<dyn std::error::Error> {
  Box::<dyn std::error::Error>::from(message)
}

fn decompile_class(scalap_args: ScalapArguments) -> R<()> {
  let parent_dotted_path = scalap_args.parent_dotted_path;
  let relative_parent_path = scalap_args.parent_relative_path;
  let class_name = scalap_args.class_name;
  let working_dir = scalap_args.working_dir;
  let target_dir = scalap_args.target_dir;
 
  let output_dir = target_dir.join(relative_parent_path.value());
  let dotted_scala_file = format!("{}.{}", parent_dotted_path.value(), class_name.value());
  let target_scala_file = format!("{}/{}.scala", output_dir.clone().to_string_lossy(), class_name.value());

  if !output_dir.is_dir() {
    fs::create_dir_all(output_dir.clone())?
  }

  // println!("scalap {} > {}", dotted_scala_file, target_scala_file);
  // println!("###> {}", output_dir.clone().to_string_lossy());
  print!("writing: {} -> ", dotted_scala_file);

  let output = 
    Command::new("scalap")
    .current_dir(working_dir)
    .arg(dotted_scala_file)
    .output()?;

    let mut output_file = fs::File::create(target_scala_file)?;
    output_file.write_all(&output.stdout)?;

    let result = 
      if output.status.success() {
        SUCCESS
      } else {
        FAILURE
      };

    print!("{}", result);
    println!();

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
