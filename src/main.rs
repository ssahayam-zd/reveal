use walkdir::{DirEntry, WalkDir};
use model::*;
use tokio::task;
use tokio::process::Command;
use tokio::io::AsyncWriteExt;
use futures::future::try_join_all;

mod model;

type AsynError = Box::<dyn std::error::Error + Send + Sync>;
type R<T> = Result<T, AsynError>;

const SUCCESS: &str = "success";
const FAILURE: &str = "failed";

#[tokio::main]
async fn main() -> R<()> {

  //TODO: Accept these params
  let working_dir = "/Users/sanjiv.sahayam/ziptemp/tmp-proto/7.273.0-4dd7dac3-SNAPSHOT";
  let target_dir = "/Users/sanjiv.sahayam/ziptemp/tmp-proto/7.273.0-4dd7dac3-SNAPSHOT-output2";

  walk_tree(
  WorkingDir::new(working_dir), 
  TargetDir::new(target_dir)
  ).await?;

  Ok(())
}

async fn walk_tree(working_dir: WorkingDir, target_dir: TargetDir) -> R<()> {
  let async_results: Vec<task::JoinHandle<_>> = 
    WalkDir::new(working_dir.clone())
      .into_iter()
      .filter_map(|e| e.ok())
      .filter(is_valid_file)
      .map(|entry|{
        // Each closure instance needs its own "owned" copies of these variables
        let working_dir_new = working_dir.clone();
        let target_dir_new = target_dir.clone();
        tokio::spawn(async move {
          let scalap_args = get_scalap_args(entry.clone(), working_dir_new, target_dir_new)?;     
          decompile_class(scalap_args).await
        })
      }).collect();

  match try_join_all(async_results).await {
    Ok(_) => Ok(()),
    Err(e) => Err(raise_error(&format!("error getting results: {:?}", e))), 
  }
}

fn get_scalap_args(entry: DirEntry, working_dir: WorkingDir, target_dir: TargetDir) -> R<ScalapArguments> {
    let p = entry.path();
    let class_name = p.file_name().ok_or(raise_error("Could not get file name"))?.to_string_lossy().replace(".class", "");
    let parent_path = p.parent().ok_or(raise_error("no parent dir"))?.to_string_lossy();
    let (_, relative_dir) = parent_path.split_once(&working_dir.to_string_lossy().as_str()).ok_or("can't detect relative dir")?;    
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

fn raise_error(message: &str) -> AsynError {
  AsynError::from(message)
}

async fn decompile_class(scalap_args: ScalapArguments) -> R<()> {
  let parent_dotted_path = scalap_args.parent_dotted_path;
  let relative_parent_path = scalap_args.parent_relative_path;
  let class_name = scalap_args.class_name;
  let working_dir = scalap_args.working_dir;
  let target_dir = scalap_args.target_dir;
 
  let output_dir = target_dir.join(relative_parent_path.value());
  let dotted_scala_file = format!("{}.{}", parent_dotted_path.value(), class_name.value());
  let target_scala_file = format!("{}/{}.scala", output_dir.clone().to_string_lossy(), class_name.value());

  if !output_dir.is_dir() {
    tokio::fs::create_dir_all(output_dir.clone()).await?
  }

  // println!("scalap {} > {}", dotted_scala_file, target_scala_file);
  // println!("###> {}", output_dir.clone().to_string_lossy());
  print!("writing: {} -> ", dotted_scala_file);

  let output = 
    Command::new("scalap")
    .current_dir(working_dir)
    .arg(dotted_scala_file)
    .output()
    .await?;

    let mut output_file = tokio::fs::File::create(target_scala_file).await?;
    output_file.write_all(&output.stdout).await?;

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
