use std::env;
use std::path::PathBuf;
use rppi_parser::{load_catalog, collect_all_recipes, Recipe};

fn usage_and_exit(program: &str) -> ! {
    eprintln!("Usage: {} <top-dir> [query]", program);
    std::process::exit(2);
}

fn matches_query(r: &Recipe, q: &str) -> bool {
    if r.repo.contains(q) { return true; }
    if r.name.contains(q) { return true; }
    //if r.schemas.iter().any(|s| s.contains(q)) { return true; }
    false
}

fn main() {
    let mut args = env::args();
    let prog = args.next().unwrap_or_else(|| "cli".to_string());
    let dir = match args.next() {
        Some(d) => PathBuf::from(d),
        None => usage_and_exit(&prog),
    };
    let query = args.next();

    let catalog = match load_catalog(&dir) {
        Ok(c) => c,
        Err(e) => { eprintln!("failed to load catalog: {}", e); std::process::exit(1); }
    };

    let mut recipes = collect_all_recipes(&catalog);
    if let Some(q) = query {
        recipes = recipes.into_iter().filter(|r| matches_query(r, &q)).collect();
    }

    for r in recipes.iter() {
        match serde_json::to_string_pretty(r) {
            Ok(s) => println!("{}", s),
            Err(_) => println!("{:?}", r),
        }
    }
}
