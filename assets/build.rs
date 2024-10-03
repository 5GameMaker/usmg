use core::panic;
use std::{
    env::{self, current_dir},
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Cursor, ErrorKind, Write},
    path::Path,
};

static ABS: &[u8] = b"$crate::Abstract";

struct Writers {
    pub includes: BufWriter<File>,
    pub abstracts: Cursor<Vec<u8>>,
    pub resources: Cursor<Vec<u8>>,
    pub resource_methods: Cursor<Vec<u8>>,
    pub overrides: Cursor<Vec<u8>>,
}

#[derive(Debug)]
enum PrefixIter<'a> {
    Prepare(&'a Prefix<'a>),
    Iteration(&'a Prefix<'a>, usize),
}
impl<'a> Iterator for PrefixIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Iteration(mut prefix, counter) => {
                if *counter == 0 {
                    None
                } else {
                    *counter -= 1;
                    for _ in 1..*counter {
                        match prefix {
                            Prefix::Empty => return None,
                            Prefix::Source(_) => return None,
                            Prefix::Extended(x, _) => prefix = x,
                        }
                    }
                    match prefix {
                        Prefix::Empty => None,
                        Prefix::Source(x) => Some(x),
                        Prefix::Extended(_, x) => Some(x),
                    }
                }
            }
            Self::Prepare(root) => {
                let mut prefix = *root;
                let mut count = 0;
                loop {
                    match prefix {
                        Prefix::Empty => {
                            *self = Self::Iteration(root, 0);
                            return None;
                        }
                        Prefix::Source(x) => {
                            *self = Self::Iteration(root, count);
                            return Some(x);
                        }
                        Prefix::Extended(x, _) => {
                            prefix = x;
                            count += 1;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
enum Prefix<'a> {
    Empty,
    Source(&'a str),
    Extended(&'a Prefix<'a>, &'a str),
}
impl<'a> Prefix<'a> {
    fn extend<'b>(&'b self, next: &'b str) -> Prefix<'b> {
        match self {
            Prefix::Empty => Prefix::Source(next),
            Prefix::Source(_) => Prefix::Extended(self, next),
            Prefix::Extended(_, _) => Prefix::Extended(self, next),
        }
    }

    #[allow(unused)]
    fn parent(&self) -> &Prefix {
        match self {
            Prefix::Empty | Prefix::Source(_) => &Prefix::Empty,
            Prefix::Extended(x, _) => x,
        }
    }

    #[allow(unused)]
    fn is_empty(&self) -> bool {
        matches!(self, Prefix::Empty)
    }

    fn parts(&self) -> PrefixIter {
        PrefixIter::Prepare(self)
    }
}

fn walk(path: &Path, file: &mut Writers, prefix: Prefix) {
    match fs::read_dir(path) {
        Ok(x) => {
            for x in x.map(|x| x.unwrap()) {
                walk(
                    &x.path(),
                    file,
                    prefix.extend(x.file_name().to_string_lossy().as_ref()),
                );
            }
        }
        Err(why) if why.kind() == ErrorKind::NotADirectory => {
            macro_rules! walk_file_ty {
                ($macro_ty:ident, $mime:expr, $ty:ident) => {{
                    let ident = {
                        let mut string = String::new();
                        for (i, x) in prefix.parts().enumerate() {
                            if i != 0 {
                                string.push('_');
                            }
                            string.push_str(x.replace('.', "_").to_lowercase().as_str());
                        }
                        string
                    };

                    // let <ident> = { let $<format> = Abstract { bytes: include_bytes!(".."),
                    // mime: "..", path: "/.." }; $<format>_trans }

                    let format = stringify!($macro_ty).as_bytes();

                    file.includes.write_all(b"let ").unwrap();
                    file.includes.write_all(ident.as_bytes()).unwrap();
                    file.includes.write_all(b"={\n\tlet $").unwrap();
                    file.includes.write_all(format).unwrap();
                    file.includes.write_all(b"=").unwrap();
                    file.includes.write_all(ABS).unwrap();
                    file.includes
                        .write_all(
                            format!(
                                "{{bytes:include_bytes!({:?}),path:\"",
                                path.to_string_lossy()
                            )
                            .as_bytes(),
                        )
                        .unwrap();

                    let rel_path = {
                        let mut string = String::new();
                        for x in prefix.parts() {
                            string.push('/');
                            string.push_str(x);
                        }
                        string
                    };

                    file.includes.write_all(rel_path.as_bytes()).unwrap();
                    file.includes.write_all(b"\",mime:\"").unwrap();
                    file.includes.write_all($mime.as_bytes()).unwrap();
                    file.includes.write_all(b"\"};\n\t$").unwrap();
                    file.includes.write_all(format).unwrap();
                    file.includes.write_all(b"_trans\n};").unwrap();

                    // let <ident> = {
                    //     let $<format>_ov = <ident>;
                    //     $<format>_ov_trans
                    // };

                    file.overrides.write_all(b"let ").unwrap();
                    file.overrides.write_all(ident.as_bytes()).unwrap();
                    file.overrides.write_all(b"={\n\tlet $").unwrap();
                    file.overrides.write_all(format).unwrap();
                    file.overrides.write_all(b"_ov=").unwrap();
                    file.overrides.write_all(ident.as_bytes()).unwrap();
                    file.overrides.write_all(b";\n\t$").unwrap();
                    file.overrides.write_all(format).unwrap();
                    file.overrides.write_all(b"_ov_trans\n};").unwrap();

                    // Resources { <ident>, }

                    file.abstracts.write_all(ident.as_bytes()).unwrap();
                    file.abstracts.write_all(b",").unwrap();

                    // struct Resources<Tex, Font> { pub <ident>: <ty>, }

                    file.resources.write_all(b"pub ").unwrap();
                    file.resources.write_all(ident.as_bytes()).unwrap();
                    file.resources.write_all(b":").unwrap();
                    file.resources
                        .write_all(stringify!($ty).as_bytes())
                        .unwrap();
                    file.resources.write_all(b",").unwrap();
                }};
            }
            match path.file_name().unwrap().to_string_lossy().split_once('.') {
                Some((_, "png")) => walk_file_ty!(png, "image/png", Tex),
                Some((_, "ttf")) => walk_file_ty!(ttf, "font/ttf", Font),
                Some((_, "sprites.csv")) => {
                    for x in BufReader::new(File::open(path).unwrap()).lines() {
                        let x = x.unwrap();
                        let mut iter = x.split(',');

                        let tex = iter.next().unwrap().replace(['.', '/'], "_");
                        let ident = {
                            let mut string = String::new();
                            for x in prefix.parts() {
                                string.push_str(x.replace('.', "_").to_lowercase().as_str());
                                string.push('_');
                            }
                            string.push_str(iter.next().unwrap());
                            string
                        };
                        let x = iter.next().unwrap();
                        let y = iter.next().unwrap();
                        let w = iter.next().unwrap();
                        let h = iter.next().unwrap();
                        if iter.next().is_some() {
                            panic!("'{}' not formatted correctly", path.display());
                        }

                        // pub fn <ident>(&self) -> Sprite<Tex> { Sprite { tex: &self.<tex>, rect:
                        // (<x>, <y>, <w>, <h>) } }

                        file.resource_methods.write_all(b"pub fn ").unwrap();
                        file.resource_methods.write_all(ident.as_bytes()).unwrap();
                        file.resource_methods
                            .write_all(b"(&self)->Sprite<Tex>{\nSprite{tex:&self.")
                            .unwrap();
                        file.resource_methods.write_all(tex.as_bytes()).unwrap();
                        file.resource_methods.write_all(b",rect:(").unwrap();
                        file.resource_methods.write_all(x.as_bytes()).unwrap();
                        file.resource_methods.write_all(b",").unwrap();
                        file.resource_methods.write_all(y.as_bytes()).unwrap();
                        file.resource_methods.write_all(b",").unwrap();
                        file.resource_methods.write_all(w.as_bytes()).unwrap();
                        file.resource_methods.write_all(b",").unwrap();
                        file.resource_methods.write_all(h.as_bytes()).unwrap();
                        file.resource_methods.write_all(b")\n}\n}").unwrap();
                    }
                }
                _ => (),
            }
        }
        Err(why) => panic!("{}", why),
    }
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("lib.rs");
    let file = File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(dest_path)
        .unwrap();
    let mut writers = Writers {
        includes: BufWriter::new(file),
        abstracts: Cursor::new(vec![]),
        resources: Cursor::new(vec![]),
        resource_methods: Cursor::new(vec![]),
        overrides: Cursor::new(vec![]),
    };

    writers
        .includes
        .write_all(
            "pub struct Sprite<'a,Tex>{pub tex:&'a Tex,pub rect:(u32,u32,u32,u32)}".as_bytes(),
        )
        .unwrap();

    writers
        .includes
        .write_all("pub struct Abstract{pub bytes:&'static[u8],pub path:&'static str,pub mime:&'static str}".as_bytes())
        .unwrap();

    writers
        .includes
        .write_all("#[macro_export]macro_rules!include_resources{($png:ident .png => $png_trans:expr,$ttf:ident .ttf => $ttf_trans:expr,".as_bytes())
        .unwrap();
    writers
        .includes
        .write_all(
            "$(+{$png_ov:ident .png=>$png_ov_trans:expr,$ttf_ov:ident .ttf=>$ttf_ov_trans:expr,})*)=>{{"
                .as_bytes(),
        )
        .unwrap();

    walk(
        &current_dir().unwrap().join("src"),
        &mut writers,
        Prefix::Empty,
    );

    let mut file = writers.includes;

    file.write_all("$(".as_bytes()).unwrap();
    file.write_all(writers.overrides.get_ref()).unwrap();
    file.write_all(")* ".as_bytes()).unwrap();

    file.write_all("$crate::Resources{".as_bytes()).unwrap();
    file.write_all(writers.abstracts.get_ref()).unwrap();
    file.write_all("}}}}".as_bytes()).unwrap();

    file.write_all("pub struct Resources<Tex,Font>{".as_bytes())
        .unwrap();
    file.write_all(writers.resources.get_ref()).unwrap();
    file.write_all("}".as_bytes()).unwrap();
    file.write_all("impl<Tex,Font>Resources<Tex,Font>{".as_bytes())
        .unwrap();
    file.write_all(writers.resource_methods.get_ref()).unwrap();
    file.write_all("}".as_bytes()).unwrap();

    println!("cargo::rerun-if-changed=src/");
}
