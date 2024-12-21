use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use colored::Colorize;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use directories::{BaseDirs, UserDirs};

#[derive(Clone)]
struct MovieFile {
    path: PathBuf,
    date: Option<String>,
    resolution: Option<String>,
    old_file_name: String,
    new_file_name: String,
}
struct WordParts {
    date: Option<(u32, bool)>,
    resolution: Option<u32>,
    list: Vec<String>,
}
impl Display for MovieFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let old_file_name = format!("{}", self.old_file_name).red();
        let new_file_name = format!("{}", self.new_file_name).green();
        let date = match &self.date {
            Some(date) => date.to_string(),
            None => "None".to_string(),
        };
        let resolution = match &self.resolution {
            Some(resolution) => format!("{resolution}p"),
            None => "None".to_string(),
        };
        let date = format!("{}", date).blue();
        let resolution = format!("{}", resolution).blue();
        let mut path = format!("{}", self.path.display().to_string()).white();
        if self.path.is_dir() {
            path = path.truecolor(255, 224, 112);
        }
        write!(
            f,
            "{} {{\n   Date: {}\n   Resolution: {}\n   Old: {}\n   New: {}\n}}\n",
            path, date, resolution, old_file_name, new_file_name
        )
    }
}

fn main() {
    let master_path = match get_master_path() {
        Some(path) => path,
        None => return,
    };

    let video_filename_extensions = [
        "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "3gp", "mpg", "mpeg", "m4v", "ts",
        "m2ts", "rm", "rmvb", "vob", "ogv", "asf", "divx", "h264", "hevc", "amv", "mxf", "f4v",
        "gxf", "mpe", "vid",
    ];
    let _subtitle_filename_extensions = [
        "srt", "ass", "ssa", "sub", "vtt", "idx", "stl", "sbv", "dfxp", "scc", "lrc", "ttml",
        "usf", "cap", "rt", "xml", "sup", "pjs",
    ];
    let mut movies: Vec<MovieFile> = vec![];
    let paths = std::fs::read_dir(&master_path).expect(&format!("{:?}", master_path));
    for path in paths {
        let dir_entry = path.unwrap();
        let path = dir_entry.path();
        let actually_old_file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let mut old_file_name = actually_old_file_name.clone();
        let mut date: Option<(u32, bool)> = None;
        let mut resolution: Option<u32> = None;
        let mut file_name_extension: Option<String> = None;
        if path.is_file() {
            if let Some(file_extension) = old_file_name.split(".").last() {
                if !video_filename_extensions.contains(&file_extension) {
                    old_file_name = (&old_file_name
                        [0..old_file_name.len() - file_extension.len() - 1])
                        .to_owned();
                } else {
                    file_name_extension = Some(file_extension.to_owned());
                }
            }
        }

        let word_parts = check_between(old_file_name, ".", &date, &resolution);
        let words1 = word_parts.list;
        date = word_parts.date;
        resolution = word_parts.resolution;
        let mut words2: Vec<Vec<String>> = vec![];
        for word in words1 {
            let word_parts = check_between(word.clone(), " ", &date, &resolution);
            date = word_parts.date;
            resolution = word_parts.resolution;
            let list = word_parts.list;
            words2.push(list);
        }
        let mut new_file_name = String::new();
        let mut should_break = false;
        for words1 in words2 {
            for word in words1 {
                if let Some(resolution) = resolution {
                    if word == format!("{}p", resolution.to_string()) {
                        should_break = true;
                    }
                }
                if let Some(date) = date {
                    if word == date.0.to_string() {
                        should_break = true;
                    }
                }
                if should_break {
                    break;
                }
                new_file_name.push_str(&word);
                new_file_name.push_str(" ");
            }
            if should_break {
                break;
            }
        }
        if let Some(date) = date {
            new_file_name.push_str(&format!("({}) ", date.0));
        }
        if let Some(resolution) = resolution {
            new_file_name.push_str(&format!("{}p ", resolution));
        }
        new_file_name.pop();
        if let Some(file_name_extension) = file_name_extension {
            new_file_name.push_str(&format!(".{}", file_name_extension));
        }
        let date = match date {
            Some(date) => Some(date.0.to_string()),
            None => None,
        };
        let resolution = match resolution {
            Some(resolution) => Some(resolution.to_string()),
            None => None,
        };
        movies.push(MovieFile {
            path: path,
            old_file_name: actually_old_file_name,
            new_file_name: new_file_name,
            date,
            resolution,
        });
    }
    clear_screen();
    execute!(std::io::stdout(), Hide).unwrap();
    println!(
        "Require confirmation for every movie? {}",
        format!("[y/n]").green()
    );
    let require_confirmation = match get_confirmation() {
        Ok(answer) => answer,
        Err(()) => return,
    };
    clear_screen();
    for movie in movies.iter_mut() {
        let mut confirmation = true;
        if movie.old_file_name != movie.new_file_name {
            let original_path = master_path.join(&movie.old_file_name);
            let mut new_path = master_path.join(&movie.new_file_name);
            if new_path.exists() {
                movie.new_file_name = format!("{}-", movie.new_file_name);
                new_path = master_path.join(&movie.new_file_name);
            }
            if require_confirmation {
                println!("{movie}");
                println!("Rename movie? {}", format!("[y/n]").green());
            }
            if require_confirmation {
                confirmation = match get_confirmation() {
                    Ok(answer) => answer,
                    Err(()) => return,
                };
            }
            if confirmation {
                if let Err(err) = std::fs::rename(&original_path, &new_path) {
                    println!(
                        "Renaming failed. Err: {err:?}\n Old file path: {:?}\nNew file path: {:?}",
                        original_path, new_path
                    );
                    return;
                }
            }
        }
        clear_screen();
    }
    execute!(std::io::stdout(), Show).unwrap();
}
fn get_master_path() -> Option<PathBuf> {
    let mut previous_path = Path::new("~/Desktop").to_path_buf();
    let mut text_file_directory: Option<PathBuf> = None;
    let mut text_file_path: Option<PathBuf> = None;
    if let Some(base_dirs) = BaseDirs::new() {
        text_file_directory = Some(base_dirs.config_dir().join("RenameMovies"));
        let text_file_directory = text_file_directory.clone().unwrap();
        std::fs::create_dir_all(&text_file_directory).unwrap();
        text_file_path = Some(text_file_directory.join("previous_path.txt"));
        if let Some(text_file_path) = text_file_path.as_ref() {
            if text_file_path.exists() {
                if let Ok(string) = std::fs::read_to_string(&text_file_path) {
                    previous_path = Path::new(&string).to_path_buf();
                    if !previous_path.exists() {
                        if let Some(user_dirs) = UserDirs::new() {
                            previous_path = match user_dirs.desktop_dir() {
                                Some(desktop_dir) => desktop_dir.to_path_buf(),
                                None => Path::new("~").to_path_buf(),
                            }
                        }
                    }
                }
            }
        }
    }
    let path = native_dialog::FileDialog::new()
        .set_location(&previous_path)
        .show_open_single_dir()
        .unwrap();
    match path {
        Some(path) => {
            if let Some(text_file_path) = text_file_path.as_ref() {
                println!("Text file path exists");
                if let Some(text_file_contents) = path.to_str() {
                    println!("Text file contents exist: {}", text_file_contents);
                    println!("Text file directory: {:?}", text_file_directory);
                    match std::fs::write(&text_file_path, text_file_contents) {
                        Ok(_) => {
                            println!("Write succeeded!");
                        }
                        Err(err) => {
                            println!("Write failed: {err:?}");
                        }
                    }
                }
            }
            return Some(path);
        }
        None => return None,
    }
}
fn clear_screen() {
    let mut stdout = std::io::stdout();
    execute!(
        stdout,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    )
    .unwrap();
    execute!(stdout, MoveTo(0, 0)).unwrap();
}
fn get_confirmation() -> Result<bool, ()> {
    enable_raw_mode().unwrap();
    loop {
        if event::poll(std::time::Duration::from_millis(500)).unwrap() {
            if let Event::Key(KeyEvent {
                code,
                modifiers,
                kind,
                ..
            }) = event::read().unwrap()
            {
                match kind {
                    KeyEventKind::Press => match code {
                        KeyCode::Char('y') => {
                            disable_raw_mode().unwrap();
                            return Ok(true);
                        }
                        KeyCode::Char('n') => {
                            disable_raw_mode().unwrap();
                            return Ok(false);
                        }
                        KeyCode::Esc => {
                            execute!(std::io::stdout(), Show).unwrap();
                            disable_raw_mode().unwrap();
                            return Err(());
                        }
                        KeyCode::Char('c') => {
                            if modifiers.contains(KeyModifiers::CONTROL) {
                                execute!(std::io::stdout(), Show).unwrap();
                                disable_raw_mode().unwrap();
                                return Err(());
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
}
fn check_between(
    input_string: String,
    split_string: &str,
    date_found: &Option<(u32, bool)>,
    resolution_found: &Option<u32>,
) -> WordParts {
    let mut list: Vec<String> = input_string
        .split(split_string)
        .map(|x| x.to_owned())
        .collect();
    let mut date: Option<(u32, bool)> = date_found.clone();
    let mut resolution: Option<u32> = resolution_found.clone();
    let mut parentheses_around_word = false;

    for word in list.iter_mut() {
        if word.len() > 0 {
            if &word[0..1] == "(" || &word[0..1] == "[" || &word[0..1] == "]" || &word[0..1] == ")"
            {
                if &word[0..1] == "(" || &word[0..1] == ")" {
                    parentheses_around_word = true;
                }
                word.remove(0);
            }
        }
        if word.len() > 0 {
            if &word[word.len() - 1..word.len()] == "("
                || &word[word.len() - 1..word.len()] == "["
                || &word[word.len() - 1..word.len()] == "]"
                || &word[word.len() - 1..word.len()] == ")"
            {
                if &word[word.len() - 1..word.len()] == "("
                    || &word[word.len() - 1..word.len()] == ")"
                {
                    parentheses_around_word = true;
                }
                word.pop();
            }
        }
        if word.len() < 4 {
            continue;
        }
        if match date {
            Some(date) => !date.1,
            None => true,
        } {
            if let Ok(number) = word.parse::<u32>() {
                date = Some((number, parentheses_around_word));
            }
        }
        if word.len() < 5 {
            continue;
        }
        if &word[word.len() - 1..word.len()] == "p" {
            if let Ok(number) = (&word[0..word.len() - 1]).parse::<u32>() {
                resolution = Some(number);
            }
        }
    }
    WordParts {
        list,
        date,
        resolution,
    }
}
