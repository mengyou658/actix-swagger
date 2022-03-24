use std::error::Error;

fn main() -> Result<(),  Box<Error>> {
    let currDir = std::env::current_dir().unwrap().to_str().unwrap().to_string();
    println!("{:?}", currDir);

    let json = currDir.clone() + "/demo/openapi.json";
    let json = "C:/Users/Administrator/Downloads/swagger.json";
    let path = std::path::Path::new(&json);

    let content = std::fs::read_to_string(&path)?;

    let format = swagg::Format::Json;

    let source_code = swagg::to_string(&content, format).unwrap();

    let code = format!("{}", source_code);
    let file = currDir + "/demo/src/api.rs";
    std::fs::write(file, code).expect("Failed to write rust code to out file");
    Ok(())
}
