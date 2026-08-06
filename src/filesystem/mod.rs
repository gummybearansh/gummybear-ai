use std::time::Instant;
use crate::error::HarnessError;

// complete FIND & REPLACE patch orchestrator - atomic - file remains uncorrupted if patch fails (LLM Hallucinates)
pub async fn process_patch (path: &str, search: &str, replace: &str) -> Result<(), HarnessError> {
    // read the file into ram 
    let file_contents = read_file(path).await?;
    

    // unpack the coordinates directly into vairables
    let Some((start, end)) = find_offsets(&file_contents, search) else {
        // if we hit None we go into the else block 
        return Err(HarnessError::PatchFailed);
    };

    // apply the patch and get the new complete string 
    let new_file_contents = apply_patch(&file_contents, start, end, replace);

    // now we have the mutated string - write it entirely to the file 
    write_file(path, &new_file_contents).await?;

    Ok(())
}

pub async fn read_file (path: &str) -> Result<String, HarnessError> {
    let start = Instant::now();

    // get file contents
    let contents = tokio::fs::read_to_string(path).await?;
    
    let duration = start.elapsed();
    println!("\n\n[FS TELEMETRY] Read {} in {:?}\n\n", path, duration);

    // return them if no error occured
    return Ok(contents);
}

pub async fn write_file(path: &str, contents: &str) -> Result<(), HarnessError> {
    let start = Instant::now();

    // write to file
    tokio::fs::write(path, contents).await?;
    
    let duration = start.elapsed();
    println!("\n\n[FS TELEMETRY] Wrote to {}, in {:?}\n\n", path, duration);

    // empty return if write was successful
    return Ok(());
}

pub fn find_offsets(source: &str, search_block: &str) -> Option<(usize, usize)> {
    let start_time = Instant::now();

    let start = source.find(search_block)?;
    // find the end from the search size directly
    let end = start + search_block.len();

    println!("\n\n[FS TELEMETRY] found string \n\n{}\n\n in {:?}\n\n", search_block, start_time.elapsed());
    return Some((start, end));
}

pub fn apply_patch(source: &str, start: usize, end: usize, replace_block: &str) -> String {
    let prefix = &source[..start];
    let suffix = &source[end..];

    let new_contents = format!("{}{}{}", prefix, replace_block, suffix);

    return new_contents;
}
