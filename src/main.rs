use walkdir::{DirEntry, WalkDir};
use model::*;

mod model;

type AsynError = Box::<dyn std::error::Error + Send + Sync>;
type R<T> = Result<T, AsynError>;

const SUCCESS: &str = "✅";
const FAILURE: &str = "☠️";

#[tokio::main]
async fn main() -> R<()> {

  //TODO: Accept these params
  let version = "7.273.0";
  let working_dir = &format!("/Users/sanjiv.sahayam/ziptemp/tmp-proto/{}-input", &version);
  let target_dir = &format!("/Users/sanjiv.sahayam/ziptemp/tmp-proto/{}-output", &version);

  // walk_tree(
  // WorkingDir::new(working_dir), 
  // TargetDir::new(target_dir)
  // )?;

  walk_tree2(
  WorkingDir::new(working_dir), 
  TargetDir::new(target_dir)
  ).await

  // Ok(())
}

// fn walk_tree(working_dir: WorkingDir, target_dir: TargetDir) -> R<()> {
//   let maybe_results: Vec<_> = 
//     WalkDir::new(working_dir.clone())
//       .into_iter()
//       .filter_map(|e| e.ok())
//       .filter(is_valid_file)
//       .map(|entry|{
//         // Each closure instance needs its own "owned" copies of these variables
//         let working_dir_new = working_dir.clone();
//         let target_dir_new = target_dir.clone();
//         // tokio::spawn(async move {
//         let scalap_args = get_scalap_args(entry.clone(), working_dir_new, target_dir_new)?;     
//         decompile_class(scalap_args)
//         // })
//       }).collect();

//   let results: Result<Vec<_>, AsynError> = maybe_results.into_iter().collect();
//   match results {
//     Ok(r) => {
//       println!("successes ========================> {}", r.len());
//       Ok(())
//     },
//     Err(e) => Err(raise_error(&format!("error getting results: {:?}", e))), 
//   }
// }

async fn walk_tree2(working_dir: WorkingDir, target_dir: TargetDir) -> R<()> {
  let args_results: R<Vec<_>> =
    WalkDir::new(working_dir.clone())
      .into_iter()
      .filter_map(|e| e.ok())
      .filter(is_valid_file)
      .map(|entry|{
        // Each closure instance needs its own "owned" copies of these variables
        let working_dir_new = working_dir.clone();
        let target_dir_new = target_dir.clone();
        get_scalap_args(entry.clone(), working_dir_new, target_dir_new)    
      }).collect();

  match args_results {
    Ok(results) => {
      let chunked_results: Vec<Vec<ScalapArguments>> =
        results
          .chunks(10)
          .map(|c| c.to_vec())
          .collect();

        execute_async(chunked_results).await
    },
    Err(e) => Err(raise_error(&format!("error getting results: {:?}", e))) 
  }
}

async fn execute_async(items: Vec<Vec<ScalapArguments>>) -> R<()> {
  use futures::{stream, StreamExt};

  let streamed = 
    stream::iter(items)
      .then(|c|{
        async {
          concurrently(c).await  
        }
      });

  let results: Vec<R<()>> = streamed.collect().await;

  let traversed: R<Vec<()>> = results.into_iter().collect();

  traversed.map(|_| ())
}

async fn concurrently(items: Vec<ScalapArguments>) -> R<()> {
  let handles: Vec<tokio::task::JoinHandle<R<()>>> = 
    items
      .into_iter()
      .map(|cx|{
        tokio::spawn ({
            decompile_class(cx)
        })
      }).collect();

  let executed_handles_result = 
    futures::future::try_join_all(handles).await;

  match executed_handles_result {
    Ok(results) => {
       let valid_results: R<Vec<()>> = results.into_iter().collect();
       valid_results.map(|_| ())
    },
    Err(e) => Err(raise_error(&format!("error joining chunks: {:?}", e))) 
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

  println!("decompile: {}", dotted_scala_file.clone());
  // println!("###> {}", output_dir.clone().to_string_lossy());

  let output = 
    tokio::process::Command::new("scalap")
    .current_dir(working_dir)
    .arg(dotted_scala_file.clone())
    .output().await?;

  println!("writing: {}", target_scala_file.clone());

  let mut output_file = tokio::fs::File::create(target_scala_file).await?;
  tokio::io::AsyncWriteExt::write_all(&mut output_file, &output.stdout).await?;
  // output_file.write_all().await?;

  let result = 
    if output.status.success() {
      SUCCESS
    } else {
      FAILURE
    };


  println!("{} {}", result, dotted_scala_file.clone());

    Ok(())
}

fn is_valid_file(entry: &DirEntry) -> bool {
  let is_dir = entry.file_type().is_dir();

  let is_class = 
    if entry.file_type().is_file() {
      let file_name_has_dollar = 
        entry
          .path()
          .file_name()
          .map(|os_str| os_str.to_string_lossy())
          .and_then(|s| s.rfind(".class"));
      
      file_name_has_dollar.map_or(false, |_| true)
    } else {
      false //not a nested class if it's not a file
    };

    !is_dir && is_class
}
