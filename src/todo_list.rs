use std::fs::File;
use std::path::PathBuf;
use std::ops::{Index, IndexMut};
use std::io::{BufWriter, Write};
use std::io;
use std::fs::read_to_string;
pub mod todo;
use todo::Todo;

#[derive(Debug,PartialEq, Clone, Default)]
pub struct TodoArray {
    pub todos: Vec<Todo>
}

impl Index<usize> for TodoArray {
    type Output = Todo;
    fn index(&self, index:usize) -> &Self::Output {
        &self.todos[index]
    }
}

impl IndexMut<usize> for TodoArray {
    fn index_mut(&mut self, index:usize) -> &mut Todo {
        &mut self.todos[index]
    }
}

pub enum TodoArrayError {
    IndexNotFound
}

impl TodoArray {
    fn new() -> Self{
        TodoArray {
            todos: Vec::new()
        }
    }

    pub fn display(&self) -> Vec<String> {
        self.todos.iter().map(|todo| todo.display()).collect()
    }

    pub fn len(&self) -> usize{
        self.todos.len()
    }

    pub fn remove(&mut self, index:usize) -> Todo{
        self.todos.remove(index)
    }

    fn push(&mut self,item:Todo) {
        self.todos.push(item)
    }

    #[inline(always)]
    fn reorder_low_high(&self, index:usize) -> (usize, usize){
        let priority = self.todos[index].comparison_priority();
        if index+1 < self.todos.len() && priority > self.todos[index+1].comparison_priority() {
            (index+1, self.todos.len()-1)
        } else {
            (0, index)
        }
    }

    pub fn reorder(&mut self, index:usize) -> usize {
        let priority = self.todos[index].comparison_priority();

        if priority < self.todos[0].comparison_priority() {
            return self.move_index(index, 0, 1)
        }

        let (low, high) = self.reorder_low_high(index);
        for middle in low..high {
            if priority < self.todos[middle+1].comparison_priority() &&
            priority >= self.todos[middle].comparison_priority() {
                return self.move_index(index, middle, 0);
            }
        }
        return self.move_index(index, high, 0);
        // return high
    }

    // pub fn reorder(&mut self, index:usize) -> Result<(), TodoArrayError> {
    //     if index > self.todos.len() {
    //         return Err(TodoArrayError::IndexNotFound);
    //     }
    //     let priority = self.todos[index].comparison_priority();
    //     if priority < self.todos[0].comparison_priority() {
    //         self.move_index(index, 0, 1)
    //     }
    //     let (mut low, mut high) = self.reorder_low_high(index);

    //     while low < high {
    //         let middle = (low + high) / 2;
    //         if priority < self.todos[middle + 1].comparison_priority()
    //             && priority >= self.todos[middle].comparison_priority()
    //         {
    //             self.move_index(index, middle, 0);
    //             return Ok(());
    //         }

    //         if priority < self.todos[middle].comparison_priority() {
    //             high = middle - 1;
    //         } else {
    //             low = middle + 1;
    //         }
    //     }
    //     // If isn't first and not in the middle, then its the last one
    //     self.move_index(index, self.todos.len()-1, 0);
    //     Ok(())
    // }

    #[inline(always)]
    fn move_index(&mut self, from: usize, to: usize, shift:usize) -> usize{

        let mut j = from;
        if from < to
        {
            for i in from..to {
                self.todos.swap(i, i+1);
                j = i+1;
            }
        } else {
            for i in (to+1-shift..from).rev() {
                self.todos.swap(i, i+1);
                j = i;
            }

        }
        return j;
        // if to == from {
        //     return;
        // }

        // let tmp = std::mem::replace(&mut self.todos[from], Default::default());

        // if to < from {
        //     self.todos.insert(to, tmp);
        //     self.todos.remove(from + 1);
        // } else {
        //     self.todos.insert(to + 1, tmp);
        //     self.todos.remove(from);
        // }
    }

    pub fn print (&self) {
        let mut i = 1;
        for todo in &self.todos {
            println!("{} - {}", i,todo.as_string());
            i+=1;
        }
    }

    #[inline(always)]
    pub fn sort (&mut self) {
        // , ascending:Option<bool>
        // let ascending = ascending.unwrap_or(false);
        self.todos.sort_by(|a, b| a.comparison_priority().cmp(&b.comparison_priority()));
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct TodoList {
    pub undone: TodoArray,
    pub done: TodoArray,
}

impl TodoList {
    pub fn new () -> Self {
        let undone = TodoArray::new();
        let done = TodoArray::new();

        TodoList {
            done,
            undone
        }
    }
    pub fn read (filename: &PathBuf) -> Self{
        let mut todo_list = Self::new();
        if !filename.is_file() {
            return todo_list
        }
        for line in read_to_string(filename).unwrap().lines() {
            let todo = match Todo::try_from(line) {
                Ok(value) => value,
                Err(..) => continue,
            };
            if todo.done() {
                todo_list.done.push(todo);
            } else {
                todo_list.undone.push(todo);
            }
        }
        return todo_list
    }

    pub fn add(&mut self, todo:Todo) {
        self.undone.push(todo);
    }

    pub fn fix_undone(&mut self) {
        for index in 0..self.undone.todos.len() {
            if self.undone.todos[index].done() {
                self.done.push(self.undone.todos.remove(index));
            }
            if index+1 >= self.undone.todos.len() {
                break;
            }
        }
    }

    pub fn write (&self, filename: &PathBuf) -> io::Result<()> {
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);
        let todos = [&self.undone.todos, &self.done.todos];

        for todo in todos.iter().flat_map(|v| v.iter()) {
            let _ = todo.dependencies.write(&todo.dependency_path());
            writeln!(writer, "{}", todo.as_string())?;
        }
        writer.flush()?;
        Ok(())
    }
}
