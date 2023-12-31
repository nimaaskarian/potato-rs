// vim:fileencoding=utf-8:foldmethod=marker
//std{{{
use std::{io::{self,Write}, path::PathBuf, ops::Add, fs::{File, remove_file}};
//}}}
// lib{{{
use scanf::sscanf;
// }}}
// mod{{{
mod note;
use crate::fileio::note_path;
use note::{Note, sha1};
use super::TodoList;
//}}}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Todo {
    pub message: String,
    note: String,
    priority: i8,
    pub dependencies: TodoList,
    dependency_name: String,
    done:bool,
}

impl Into<String> for &Todo {
    fn into(self) -> String{
        let done_str = if self.done() {"-"} else {""};
        let mut note_str = match self.note.as_str() {
            "" => String::new(),
            _ => format!(">{}", self.note),
        };
        if self.has_dependency() {
            note_str = format!(">{}", self.dependency_name);
        }
        format!("[{done_str}{}]{note_str} {}", self.priority, self.message)
    }
}

#[derive(Debug, PartialEq)]
pub enum TodoError {
    ReadFailed,
    NoteEmpty,
    AlreadyExists,
    DependencyCreationFailed,
}

impl TryFrom<String> for Todo {
    type Error = TodoError;

    fn try_from(s:String) -> Result<Todo, TodoError>{
        Todo::try_from(s.as_str())
    }
}

impl TryFrom<&str> for Todo {
    type Error = TodoError;

    fn try_from(s:&str) -> Result<Todo, TodoError>{
        let mut message = String::new();
        let mut note = String::new();
        let mut todo = String::new();
        let mut priority_string:String = String::new();

        if sscanf!(s,"[{}]>{}.todo {}", priority_string, todo, message).is_err() {
            if sscanf!(s,"[{}]>{} {}", priority_string, note, message).is_err() {
                if sscanf!(s,"[{}] {}", priority_string, message).is_err() {
                    return Err(TodoError::ReadFailed);
                }
            }
        }
        let mut dependency_name = String::new();
        let mut dependencies = TodoList::new();
        if todo != "" {
            dependency_name = Self::static_dependency_name(&todo);
            dependencies = TodoList::read(&note_path(&dependency_name).unwrap());
        }

        let done = priority_string.chars().nth(0).unwrap() == '-';
        let mut priority:i8 = priority_string.parse().unwrap();

        if done {
            priority*=-1;
        }
        Ok(Todo {
            dependency_name,
            dependencies,
            message,
            note,
            priority,
            done,
        })
    }
}

impl Todo {
    pub fn new(message:String, priority:i8) -> Self {
        Todo {
            dependency_name: String::new(),
            note: String::new(),
            dependencies: TodoList::new(),
            message,
            priority: Todo::fixed_priority(priority),
            done: false,
        }
    }

    fn static_dependency_name(name:&String) -> String {
        format!("{name}.todo")
    }

    // fn dependency_name(&self) -> &String {
    //     if self.dependency_name.is_empty() {
    //         let hash = self.hash();
    //         &Self::static_dependency_name(&hash)
    //     } else {
    //         &self.dependency_name
    //     }
    // }

    pub fn dependency_path(&self) -> PathBuf {
        note_path(&self.dependency_name).unwrap()
    }

    pub fn remove_note(&mut self) -> io::Result<()>{
        remove_file(note_path(&self.note).unwrap())?;
        self.note = String::new();
        Ok(())
    }

    pub fn add_dependency(&mut self) -> Result<(), TodoError>{
        if self.has_dependency() {
            return Err(TodoError::AlreadyExists)
        }
        self.remove_note();
        self.dependency_name = Self::static_dependency_name(&self.hash());
        if File::create(self.dependency_path()).is_err() {
            return Err(TodoError::DependencyCreationFailed)
        }

        self.dependencies = TodoList::read(&self.dependency_path());
        Ok(())
    }

    pub fn has_dependency(&self) -> bool {
        return !self.dependency_name.is_empty();
        // self.dependencies.undone.len() != 0
    }

    pub fn done(&self) -> bool {
        let done_len = self.dependencies.done.len();
        if self.has_dependency() && done_len != 0{
            return self.dependencies.undone.len() == 0;
        }
        return self.done
    }

    pub fn display(&self) -> String {
        let done_string = if self.done() {
            "x"
        } else {
            " "
        };
        let note_string = if self.note != "" {
            ">"
        } else if self.has_dependency() {
            "-"
        }
        else {
            " "
        };
        format!("[{done_string}] [{}]{note_string}{}", self.priority, self.message)
    }

    pub fn remove_dependency(&mut self) {
        remove_file(self.dependency_path());
        self.dependency_name = String::new();
        self.dependencies = TodoList::new();
    }

    pub fn add_note(&mut self)-> io::Result<()>{
        let note = Note::from_editor()?;

        self.set_note(note);
        Ok(())
    }

    pub fn set_note(&mut self, note:Note) {
        self.remove_dependency();
        self.note = note.hash();
        note.save().expect("Note saving failed");
    }

    pub fn edit_note(&mut self)-> io::Result<()>{
        let mut note = Note::from_hash(&self.note)?;
        note.edit_with_editor()?;
        self.note = note.hash();
        note.save().expect("Note saving failed");
        Ok(())
    }

    pub fn get_note(&self) -> String {
        match Note::from_hash(&self.note) {
            Err(_) => return String::new(),
            Ok(note) => note.content()
        }
    }

    pub fn set_message(&mut self, message:String) {
        self.message = message;
    }

    pub fn hash(&self) -> String{
        sha1(&format!("{} {}", self.priority, self.message))
    }

    pub fn toggle_done(&mut self) {
        self.done = !self.done;
    }

    pub fn decrease_priority(&mut self) {
        if self.comparison_priority() < 9 {
            self.priority+=1
        } else {
            self.priority=0
        }
    }

    pub fn increase_priority(&mut self) {
        if self.comparison_priority() > 1 {
            self.priority=self.comparison_priority()-1
        } else {
            self.priority=1
        }
    }

    pub fn set_priority(&mut self, add:i8) {
        self.priority = add;
        self.fix_priority();
    }

    fn fix_priority(&mut self) {
        self.priority = Todo::fixed_priority(self.priority)
    }

    #[inline(always)]
    pub fn comparison_priority(&self) -> i8{
        if self.priority == 0 {10} else {self.priority}
    }

    fn fixed_priority(priority: i8) -> i8 {
        match priority {
            10.. => 0,
            0 => 0,
            ..=0 => 1,
            _ => priority
        }
    }

    pub fn as_string(&self) -> String{
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use crate::fileio::append_home_dir;

    use super::*;
    use std::fs;

    #[test]
    fn test_todo_into_string() {
        let mut todo = Todo::new("Test".to_string(), 1);
        todo.set_note(Note::new("Note".to_string()));

        let expected = "[1]>2c924e3088204ee77ba681f72be3444357932fca Test";
        let result: String = (&todo).into();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_try_from_string() {
        let input = "[1]>2c924e3088204ee77ba681f72be3444357932fca Test";
        let expected = Ok(Todo {
            dependency_name: String::new(),
            message: "Test".to_string(),
            note: "2c924e3088204ee77ba681f72be3444357932fca".to_string(),
            priority: 1,
            dependencies: TodoList::new(),
            done: false,
        });

        let result: Result<Todo, TodoError> = Todo::try_from(input.to_string());

        assert_eq!(result, expected);
    }

    #[test]
    fn test_new_todo() {
        let message = "New Todo";
        let priority = 2;

        let todo = Todo::new(message.to_string(), priority);

        assert_eq!(todo.message, message);
        assert_eq!(todo.note, String::new());
        assert_eq!(todo.priority, 2);
        assert_eq!(todo.dependencies, TodoList::new());
        assert_eq!(todo.dependency_name, String::new());
        assert_eq!(todo.done, false);
    }

    #[test]
    fn test_static_dependency_name() {
        let name = "my_dep".to_string();
        let expected = "my_dep.todo".to_string();

        let result = Todo::static_dependency_name(&name);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_static_dependency_path() {
        let name = "my_dep".to_string();
        let expected = PathBuf::from(append_home_dir(".local/share/calcurse/notes/my_dep.todo"));

        let result = note_path(&Todo::static_dependency_name(&name)).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_dependency_name() {
        let mut todo = Todo::new("Test".to_string(), 1);
        let expected = "900a80c94f076b4ee7006a9747667ccf6878a72b.todo";

        todo.add_dependency();

        let result = todo.dependency_name;

        assert_eq!(result, expected);
    }

    #[test]
    fn test_remove_note() {
        let mut todo = Todo::new("Test".to_string(), 1);
        todo.set_note(Note::new("Note".to_string()));

        todo.remove_note();

        assert_eq!(todo.note, String::new());
    }

    #[test]
    fn test_add_dependency() {
        let mut todo = Todo::new("Test".to_string(), 1);

        todo.add_dependency();

        assert!(todo.has_dependency());
    }

    #[test]
    fn test_remove_dependency() {
        let mut todo = Todo::new("Test".to_string(), 1);
        todo.add_dependency();

        todo.remove_dependency();

        assert!(!todo.has_dependency());
    }

    #[test]
    fn test_toggle_done() {
        let mut todo = Todo::new("Test".to_string(), 1);

        todo.toggle_done();
        assert_eq!(todo.done(), true);

        todo.toggle_done();
        assert_eq!(todo.done(), false);
    }

    #[test]
    fn test_from_string() {
        let input1 = "[1]>1BE348656D84993A6DF0DB0DECF2E95EF2CF461c.todo Read for exams";
        let todo = Todo::try_from(input1).unwrap();

        let expected = Todo {
            dependency_name: "1BE348656D84993A6DF0DB0DECF2E95EF2CF461c.todo".to_string(),
            message: "Read for exams".to_string(),
            note: String::new(),
            priority: 1,
            dependencies: TodoList::new(),
            done: false,
        };
        assert_eq!(todo, expected);
        assert_eq!(todo.dependency_path(), note_path(&"1BE348656D84993A6DF0DB0DECF2E95EF2CF461c.todo".to_string()).unwrap());
    }
}
