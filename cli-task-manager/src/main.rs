use clap::{Parser, Subcommand};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::process;

const TASK_FILE: &str = "tasks.json";

#[derive(Parser)]
#[command(name = "CLI Task Manager")]
#[command(about = "A simple CLI task manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a task
    Add {
        #[arg(short, long)]
        description: String,
    },
    /// List all tasks
    List,
    /// Remove a task by ID
    Remove {
        #[arg(short, long)]
        id: u32,
    },
    /// Edit a task description by ID
    Edit {
        #[arg(short, long)]
        id: u32,
        #[arg(short, long)]
        description: String,
    },
    /// Toggle the completed state of a task by ID
    Toggle {
        #[arg(short, long)]
        id: u32,
    },
    /// Fuzzy search tasks by description, ID, or status
    Search {
        #[arg(short, long)]
        query: String,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Task {
    id: u32,
    description: String,
    completed: bool,
}

fn load_tasks() -> Vec<Task> {
    let file = File::open(TASK_FILE);
    match file {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .expect("Could not read file");
            serde_json::from_str(&contents).unwrap_or_else(|_| Vec::new())
        }
        Err(_) => Vec::new(), // Return empty if file doesn't exist
    }
}

fn save_tasks(tasks: &Vec<Task>) {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(TASK_FILE)
        .expect("Could not open file");
    serde_json::to_writer_pretty(file, tasks).expect("Could not write to file");
}

fn add_task(description: String) {
    let mut tasks = load_tasks();
    let next_id = tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    tasks.push(Task {
        id: next_id,
        description,
        completed: false,
    });
    save_tasks(&tasks);
    println!("Task added.");
}

fn list_tasks() {
    let tasks = load_tasks();
    if tasks.is_empty() {
        println!("No tasks available.");
    } else {
        println!("Tasks:");
        for task in tasks {
            println!(
                "{}. {} - {}",
                task.id,
                task.description,
                if task.completed {
                    "Completed"
                } else {
                    "Pending"
                }
            );
        }
    }
}

fn remove_task(id: u32) {
    let mut tasks = load_tasks();
    if let Some(index) = tasks.iter().position(|t| t.id == id) {
        tasks.remove(index);
        save_tasks(&tasks);
        println!("Task removed.");
    } else {
        eprintln!("Task with id {} not found.", id);
        process::exit(1);
    }
}

fn toggle_task_completed(id: u32) {
    let mut tasks = load_tasks();
    if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
        task.completed = !task.completed;
        let task_description = task.description.clone();
        let task_status = if task.completed {
            "Completed"
        } else {
            "Pending"
        };
        save_tasks(&tasks);
        println!("Task '{}' is now {}.", task_description, task_status);
    } else {
        eprintln!("Task with id {} not found.", id);
        process::exit(1);
    }
}

fn edit_task(id: u32, new_description: String) {
    let mut tasks = load_tasks();
    if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
        task.description = new_description;
        save_tasks(&tasks);
        println!("Task with ID {} was updated.", id);
    } else {
        eprintln!("Task with id {} not found.", id);
        process::exit(1);
    }
}

fn fuzzy_search(query: String) {
    let tasks = load_tasks();
    let matcher = SkimMatcherV2::default();
    let mut found = false;

    for task in tasks {
        let task_str = format!(
            "{} {} {}",
            task.id,
            task.description,
            if task.completed {
                "Completed"
            } else {
                "Pending"
            }
        );
        if matcher.fuzzy_match(&task_str, &query).is_some() {
            println!(
                "{}. {} - {}",
                task.id,
                task.description,
                if task.completed {
                    "Completed"
                } else {
                    "Pending"
                }
            );
            found = true;
        }
    }

    if !found {
        println!("No tasks matched the query.");
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Add { description } => add_task(description.clone()),
        Commands::List => list_tasks(),
        Commands::Remove { id } => remove_task(*id),
        Commands::Toggle { id } => toggle_task_completed(*id),
        Commands::Edit { id, description } => edit_task(*id, description.clone()),
        Commands::Search { query } => fuzzy_search(query.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn reset_task_file() {
        if Path::new(TASK_FILE).exists() {
            fs::remove_file(TASK_FILE).expect("Failed to reset task file");
        }
    }

    #[test]
    fn test_add_task() {
        reset_task_file();
        add_task("Test task 1".to_string());
        let tasks = load_tasks();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].description, "Test task 1");
        assert_eq!(tasks[0].completed, false);
    }

    #[test]
    fn test_add_multiple_tasks() {
        reset_task_file();
        add_task("Task 1".to_string());
        add_task("Task 2".to_string());
        let tasks = load_tasks();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].description, "Task 1");
        assert_eq!(tasks[1].description, "Task 2");
        assert_eq!(tasks[0].id, 1);
        assert_eq!(tasks[1].id, 2);
    }

    #[test]
    fn test_remove_task() {
        reset_task_file();
        add_task("Task 1".to_string());
        add_task("Task 2".to_string());
        remove_task(1);
        let tasks = load_tasks();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].description, "Task 2");
    }

    #[test]
    fn test_remove_invalid_task() {
        reset_task_file();
        add_task("Task 1".to_string());
        let result = std::panic::catch_unwind(|| {
            remove_task(999);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_toggle_task_completed() {
        reset_task_file();
        add_task("Task 1".to_string());
        toggle_task_completed(1);
        let tasks = load_tasks();
        assert_eq!(tasks[0].completed, true);
        toggle_task_completed(1);
        let tasks = load_tasks();
        assert_eq!(tasks[0].completed, false);
    }

    #[test]
    fn test_edit_task() {
        reset_task_file();
        add_task("Task 1".to_string());
        edit_task(1, "Updated Task".to_string());
        let tasks = load_tasks();
        assert_eq!(tasks[0].description, "Updated Task");
    }

    #[test]
    fn test_fuzzy_search() {
        reset_task_file();
        add_task("Write documentation".to_string());
        add_task("Fix bug".to_string());
        fuzzy_search("doc".to_string());
        fuzzy_search("2".to_string());
    }
}
