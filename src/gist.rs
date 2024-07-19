use io::Write;
use std::io;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::str::FromStr;

pub struct Gist {
    name: String,
    pub hash: Option<String>,
}

impl Gist {
    pub fn new_with_name(name: &str) -> Gist {
        Gist {
            name: String::from_str(name).unwrap(),
            hash: None,
        }
    }

    pub fn get_hash(self: &mut Gist) -> Option<String> {
        let gist_list_output = Command::new("gh")
            .args(&["gist", "list"])
            .output()
            .expect("error on gist list command");

        match String::from_utf8(gist_list_output.stdout.to_vec())
            .unwrap()
            .split("\n")
            .into_iter()
            .find(|l| l.contains(self.name.as_str()))
        {
            Some(l) => {
                let hash = Some(
                    String::from_str(
                        l.split_once("\t")
                            .expect("error getting gist hash from string")
                            .0,
                    )
                    .unwrap(),
                );

                self.hash = hash;
                self.hash.clone()
            }
            None => None,
        }
    }

    pub fn content(self: &Gist) -> Result<Vec<u8>, io::Error> {
        assert!(self.hash.is_some(), "edit called on a Gist with no hash");

        match Command::new("gh")
            .args(&["gist", "view", &self.hash.clone().unwrap()])
            .output()
        {
            Ok(output) => Ok(output.stdout),
            Err(err) => Err(err),
        }
    }

    pub fn create(self: Gist, content: Vec<u8>) -> Result<ExitStatus, io::Error> {
        let mut out = Command::new("gh")
            .stdin(Stdio::piped())
            .args(&["gist", "create", "--public", "--filename", &self.name])
            .spawn()
            .expect("error spawning `gist create` command");

        let mut stdin = out
            .stdin
            .take()
            .expect("error getting `gist create` stdin input");

        let f = content.to_vec();
        std::thread::spawn(move || {
            stdin
                .write_all(f.as_slice())
                .expect("error writing to `gist create` stdin");
        });

        out.wait()
    }

    pub fn edit(self: Gist, new_file_path: String) -> io::Result<Output> {
        assert!(self.hash.is_some(), "edit called on a Gist with no hash");

        Command::new("gh")
            .stdin(Stdio::piped())
            .args(&["gist", "edit", &self.hash.unwrap(), new_file_path.as_str()])
            .output()
    }
}
