use std::path::PathBuf;

fn read_files_in_dir(dir: &str) -> Vec<String> {
    let mut files = vec![];

    for dir in std::fs::read_dir(dir).expect("Unable to read dir") {
        let dir = dir.expect("Unable to read entry");

        let meta = dir.metadata().expect("Unable to read file metadata");
        if !meta.is_file() {
            continue;
        }

        let file_name = dir.file_name();
        let file_name = file_name.to_string_lossy().to_string();

        files.push(file_name);
    }
    files
}

fn generate_js_files(pwd: &str) {
    let files = read_files_in_dir("static/js");

    let mut path: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    path.push("js.generated.rs");

    let mut contents = String::new();
    contents.push_str(&format!(
        "pub(super) static RESOURCES: [(&'static str, Js); {}] = [\n",
        files.len()
    ));

    for file in files {
        contents.push_str(&format!(
            "(\"{file}\", Js::new(include_str!(\"{pwd}/static/js/{file}\"))),\n"
        ));
    }
    contents.push_str(&format!("];\n"));

    std::fs::write(path, contents).unwrap();
}

fn generate_css_files(pwd: &str) {
    let files = read_files_in_dir("static/css");

    let mut path: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    path.push("css.generated.rs");

    let mut contents = String::new();
    contents.push_str(&format!(
        "pub(super) static RESOURCES: [(&'static str, Css); {}] = [\n",
        files.len()
    ));

    for file in files {
        contents.push_str(&format!(
            "(\"{file}\", Css::new(include_str!(\"{pwd}/static/css/{file}\"))),\n"
        ));
    }
    contents.push_str(&format!("];\n"));

    std::fs::write(path, contents).unwrap();
}

fn generate_html_files(pwd: &str) {
    let files = read_files_in_dir("html");

    let mut path: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    path.push("html.generated.rs");

    let mut contents = String::new();
    contents.push_str(&format!(
        "pub(super) static RESOURCES: [(&'static str, Html); {}] = [\n",
        files.len()
    ));

    for file in files {
        contents.push_str(&format!(
            "(\"{file}\", Html::new(include_str!(\"{pwd}/html/{file}\"))),\n"
        ));
    }
    contents.push_str(&format!("];\n"));

    std::fs::write(path, contents).unwrap();
}
fn main() {
    let pwd = std::env::current_dir()
        .expect("Unable to get current dir")
        .to_string_lossy()
        .to_string();

    generate_js_files(&pwd);
    generate_css_files(&pwd);
    generate_html_files(&pwd);

    println!("cargo::rerun-if-changed=static");
    println!("cargo::rerun-if-changed=html");
    println!("cargo::rerun-if-changed=build.rs");
}
