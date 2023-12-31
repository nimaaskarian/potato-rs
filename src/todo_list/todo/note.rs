use std::fs::{File, remove_file};
use std::io::{self, Write};
use sha1::{Sha1, Digest};
use std::process::Command;
use std::path::{PathBuf};
use std::env;
use crate::fileio::{note_path, temp_note_path, file_content};

pub struct Note {
    content: String,
    hash: String,
}

#[inline(always)]
pub fn open_temp_editor(content:String, path:&PathBuf) -> io::Result<String>{
    let mut file = File::create(path)?;
    write!(file, "{content}")?;
    let editor = if cfg!(windows) {
        String::from("notepad")
    } else {
         match env::var("EDITOR") {
            Ok(editor) => editor,
            Err(_) => String::from("vi")
        }
    };
    Command::new(editor).arg(path.clone()).status().expect("Couldn't open the editor.");
    let content = file_content(path)?;
    remove_file(path).unwrap();
    Ok(content)
}

impl Note {
    pub fn from_editor()-> io::Result<Self> {

        let content = open_temp_editor(String::new(), &temp_note_path())?;

        Ok(Note::new(content))
    }

    pub fn content(&self) -> String {
        self.content.clone()
    }

    pub fn from_hash(hash:&String) -> io::Result<Self> {
        let content = file_content(&note_path(&hash).expect("Unable to get the note's path."))?;
        Ok(Note::new(content))
    }

    fn path(&self) -> PathBuf {
        note_path(&self.hash).expect("Unable to get the note's path.")
    }

    pub fn new(content:String)-> Self {
        let hash = sha1(&content);

        Note {
            content,
            hash,
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let mut file = File::create(self.path().as_os_str())?;
        file.write_all(self.content.as_bytes())?;

        Ok(())
    }

    pub fn remove_file(&self) {
        remove_file(self.path()).unwrap();
    }

    pub fn edit_with_editor(&mut self) -> io::Result<()> {
        self.content = open_temp_editor(self.content.clone(), &temp_note_path())?;
        self.remove_file();
        self.hash = sha1(&self.content);
        Ok(())
    }

    pub fn hash(&self) -> String {
        self.hash.clone()
    }
}

pub fn sha1(str:&String) -> String{
    let mut hasher = Sha1::new();
    hasher.update(str);
    let result = hasher.finalize();
    format!("{:x}", result)
}
