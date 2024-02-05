use std::env;
use std::path::Path;
use std::fs::{create_dir, metadata};
use std::io::{self, Write, Read, copy};
use std::fs::{self, File};
use std::time::UNIX_EPOCH;


fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap(); // Assure que le prompt $ est affiché avant de bloquer pour l'entrée

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF ou Ctrl+D
            Ok(_) => {
                // Traitement de la commande
                execute_command(input.trim());
            },
            Err(error) => eprintln!("Error: {}", error),
        }
    }
}

fn mv(source: &Path, destination: &Path) -> io::Result<()> {
    // Construit le nouveau chemin de destination pour inclure le répertoire source
    let final_destination = if destination.is_dir() {
        destination.join(source.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid source name"))?)
    } else {
        destination.to_path_buf()
    };

    // Déplace le répertoire source vers le nouvel emplacement
    fs::rename(source, &final_destination)?;

    println!("Moved {:?} to {:?}", source, final_destination);

    Ok(())
}




fn execute_command(input: &str) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let command = parts.get(0).unwrap_or(&"");
    let args = &parts[1..];

    match *command {
        "mv" => {
            if args.len() != 2 {
                eprintln!("Usage: mv <source> <destination>");
            } else {
                let source = Path::new(args[0]);
                let destination = Path::new(args[1]);
                if let Err(e) = mv(source, destination) {
                    eprintln!("mv error: {}", e);
                }
            }
        },
        // Ajoutez d'autres commandes ici, par exemple:
        "cd" => cd(args),
        "echo" => echo(args),
        "pwd" => pwd(),
        "ls" => ls(args),
        "mkdir" => mkdir(args),
        "cat" => cat(args),
        "rm" => {
            if let Err(e) = rm(args) {
                eprintln!("rm error: {}", e);
            }
        },        "cp" => cp(args),
        "exit" => std::process::exit(0),
        _ => eprintln!("Command '{}' not found", command),
    }
}


fn mkdir(args: &[&str]) {
    if args.is_empty() {
        eprintln!("mkdir: missing operand");
        return;
    }

    for path in args {
        if let Err(e) = create_dir(path) {
            eprintln!("mkdir: cannot create directory '{}': {}", path, e);
        }
    }
}


fn cat(args: &[&str]) {
    if args.is_empty() {
        eprintln!("cat: missing operand");
        return;
    }

    for path in args {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("cat: {}: {}", path, e);
                continue;
            },
        };

        let mut contents = String::new();
        if let Err(e) = file.read_to_string(&mut contents) {
            eprintln!("cat: error reading {}: {}", path, e);
            continue;
        }

        print!("{}", contents);
    }
}

fn rm(args: &[&str]) -> io::Result<()> {
    if args.len() < 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Not enough arguments"));
    }

    let option = args[0];
    let path = Path::new(args[1]);

    match option {
        "-r" => {
            if path.is_dir() {
                // Suppression récursive du dossier
                fs::remove_dir_all(path)?;
                println!("Removed directory: {:?}", path);
            } else {
                return Err(io::Error::new(io::ErrorKind::Other, "Specified path is not a directory"));
            }
        },
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid option")),
    }

    Ok(())
}

fn cp(args: &[&str]) {
    if args.len() != 2 {
        eprintln!("cp: missing operand");
        return;
    }

    let source_path = Path::new(&args[0]);
    let mut destination_path = Path::new(&args[1]).to_path_buf();

    if let Ok(metadata) = fs::metadata(&destination_path) {
        if metadata.is_dir() {
            if let Some(filename) = source_path.file_name() {
                destination_path.push(filename);
            } else {
                eprintln!("cp: invalid source path");
                return;
            }
        }
    }

    match (File::open(&source_path), File::create(&destination_path)) {
        (Ok(mut src), Ok(mut dst)) => {
            if let Err(e) = copy(&mut src, &mut dst) {
                eprintln!("cp: error copying from {:?} to {:?}: {}", source_path, destination_path, e);
            }
        },
        (Err(e), _) => eprintln!("cp: error opening source file '{:?}': {}", source_path, e),
        (_, Err(e)) => eprintln!("cp: error creating destination file '{:?}': {}", destination_path, e),
    }
}



fn cd(args: &[&str]) {
    if args.len() > 0 {
        if let Err(e) = env::set_current_dir(&Path::new(args[0])) {
            eprintln!("cd: {}", e);
        }
    } else {
        eprintln!("cd: missing argument");
    }
}

fn echo(args: &[&str]) {
    println!("{}", args.join(" "));
}

fn pwd() {
    if let Ok(path) = env::current_dir() {
        println!("{}", path.display());
    } else {
        eprintln!("pwd: failed to get current directory");
    }
}

fn ls(args: &[&str]) {
    let mut path = ".";
    let mut show_all = false; // Correspond à l'option -a
    let mut long_format = false; // Correspond à l'option -l
    let mut classify = false; // Correspond à l'option -F

    for arg in args {
        match *arg {
            "-l" => long_format = true,
            "-a" => show_all = true,
            "-f" => classify = true,
            _ => path = arg,
        }
        
    }

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries.filter_map(Result::ok) {
                let file_name = entry.file_name().to_string_lossy().into_owned();
                // Pour -a, ignorer les fichiers commençant par un '.' sauf si -a est spécifié
                if !show_all && file_name.starts_with('.') {
                    continue;
                }

                let file_type = entry.file_type().ok().map(|ft| {
                    if ft.is_dir() { "/" } else if ft.is_symlink() { "@" } else if ft.is_file() { "*" } else { "" }
                }).unwrap_or("");

                if long_format {
                    let meta = metadata(entry.path()).ok();
                    let len = meta.clone().map(|m| m.len()).unwrap_or(0);
                      let modified = meta.and_then(|m| m.modified().ok())
                        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                        .map(|duration| duration.as_secs())
                        .unwrap_or(0);
                    println!("{:10} {} {}", len, modified, file_name);
                } else {
                    print!("{}{}", file_name, if classify { file_type } else { "" });
                    if !long_format {
                        print!(" ");
                    }
                }
            }
            if !long_format {
                println!(); // Ajoute un saut de ligne après la liste si pas en format long
            }
        },
        Err(e) => eprintln!("ls: cannot access '{}': {}", path, e),
    }
}

