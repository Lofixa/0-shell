use std::env;
use std::path::Path;
use std::fs::create_dir;
use std::io::{self, Write, Read, copy};
use std::fs::{self, remove_file, remove_dir_all, File, rename};

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

fn mv(source: &Path, destination: &Path) -> std::io::Result<()> {
    if destination.is_dir() {
        let entries = fs::read_dir(source)?;
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let dest_path = destination.join(&file_name);
            if dest_path.exists() {
                // Si le fichier existe déjà dans le répertoire de destination
                // Décider de la politique de gestion des conflits : écraser, renommer, etc.
                // Pour cet exemple, on va supprimer le fichier de destination avant de déplacer
                // Attention : cette opération est destructive et doit être utilisée avec prudence
                if dest_path.is_dir() {
                    // Si c'est un dossier, supprimer récursivement
                    fs::remove_dir_all(&dest_path)?;
                } else {
                    // Si c'est un fichier, le supprimer
                    fs::remove_file(&dest_path)?;
                }
            }
            // Déplacer (ou renommer) le fichier ou le dossier vers le répertoire de destination
            rename(entry.path(), dest_path)?;
        }
        // Supprimer le répertoire source s'il est maintenant vide
        fs::remove_dir(source)?;
    } else {
        // Si le chemin de destination n'est pas un répertoire, essayer de le renommer directement
        // Cela couvre le cas de renommage ou de déplacement dans un nouveau chemin
        if destination.exists() {
            // Gérer le cas où le fichier de destination existe déjà
            return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Destination exists"));
        }
        rename(source, destination)?;
    }
    Ok(())
}




fn mv_wrapper(args: &[&str]) -> std::io::Result<()> {
    if args.len() != 2 {
        eprintln!("Usage: mv <source> <destination>");
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid number of arguments"));
    }

    let source = Path::new(args[0]);
    let destination = Path::new(args[1]);
    mv(source, destination)
}

fn execute_command(input: &str) {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    let command = parts[0];
    let args = &parts[1..];

    match command {
        "cd" => cd(args),
        "echo" => echo(args),
        "pwd" => pwd(),
        "ls" => ls(args),
        "mkdir" => mkdir(args),
        // Remplacez "mv" par "mv_wrapper" pour utiliser le wrapper adapté
        "mv" => {
            if let Err(e) = mv_wrapper(args) {
                eprintln!("mv error: {}", e);
            }
        },
        "cat" => cat(args),
        "rm" => rm(args),
        "cp" => cp(args),
        // Ajoutez d'autres commandes ici
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

fn rm(args: &[&str]) {
    if args.is_empty() {
        eprintln!("rm: missing operand");
        return;
    }

    for path in args {
        let metadata = fs::metadata(path);
        if metadata.is_ok() && metadata.unwrap().is_dir() {
            if let Err(e) = remove_dir_all(path) {
                eprintln!("rm: failed to remove '{}': {}", path, e);
            }
        } else {
            if let Err(e) = remove_file(path) {
                eprintln!("rm: failed to remove '{}': {}", path, e);
            }
        }
    }
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
    let path = if args.is_empty() { "." } else { args[0] };

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("{}", entry.file_name().to_string_lossy());
                }
            }
        },
        Err(e) => eprintln!("ls: cannot access '{}': {}", path, e),
    }
}

