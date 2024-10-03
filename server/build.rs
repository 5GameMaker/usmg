use std::{
    env::{self, current_dir},
    fs::{self, File},
    io::Write,
    path::Path,
};

const EXT2MIME: &[(&str, &str)] = &[
    ("html", "text/html"),
    ("js", "text/javascript"),
    ("module.wasm", "application/wasm"),
];

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("client_files.rs");
    let mut file = File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(dest_path)
        .unwrap();

    file.write_all(b"{").unwrap();

    for ent in fs::read_dir("../web/dist/").unwrap().map(|x| x.unwrap()) {
        let filename_os = ent.file_name();
        let filename = filename_os.to_string_lossy();
        let ext = filename.split_once('.').unwrap().1;
        let mime = EXT2MIME
            .iter()
            .find(|x| x.0 == ext)
            .map(|x| x.1)
            .unwrap_or_else(|| panic!("Extension '{ext}' is not supported"));

        file.write_all(b"if req.uri().path()==\"/client/").unwrap();
        file.write_all(filename.as_bytes()).unwrap();
        file.write_all(b"\"{return Ok(Response::builder().header(\"Content-Type\",\"")
            .unwrap();
        file.write_all(mime.as_bytes()).unwrap();
        file.write_all(b"\").body(Full::new(Bytes::from_static(include_bytes!(")
            .unwrap();
        file.write_all(
            format!(
                "{:?}",
                current_dir().unwrap().join(ent.path()).to_string_lossy()
            )
            .as_bytes(),
        )
        .unwrap();
        file.write_all(b")))).unwrap());}\n").unwrap();
    }

    file.write_all(b"}").unwrap();

    println!("cargo::rerun-if-changed=../web/src/");
    println!("cargo::rerun-if-changed=../web/dist/");
}
