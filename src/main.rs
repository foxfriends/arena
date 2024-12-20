use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;
use wasmer::{imports, Imports, Instance, Module, Store};

#[derive(clap::ValueEnum, Clone)]
enum Game {
    Digger,
}

#[derive(Parser)]
struct Args {
    game: Game,
    #[arg(long, short)]
    bots_dir: Option<PathBuf>,
}

trait Arena: Plugin {
    fn imports(&self) -> Imports {
        imports! {}
    }
}

struct Digger;

impl Arena for Digger {}

impl Plugin for Digger {
    fn build(&self, app: &mut App) {}
}

struct Bot {
    name: String,
    instance: Instance,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let bots_dir = args
        .bots_dir
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let mut store = Store::default();
    let mut bots = vec![];

    let arena = match args.game {
        Game::Digger => Digger,
    };

    for file in std::fs::read_dir(&bots_dir)? {
        let file = file?;
        if file.path().extension().and_then(|s| s.to_str()) != Some("wasm") {
            continue;
        }
        let module = Module::from_file(&store, file.path())?;
        let instance = Instance::new(&mut store, &module, &arena.imports())?;
        bots.push(Bot {
            name: file.file_name().to_string_lossy().into_owned(),
            instance,
        })
    }

    let mut app = App::new();
    app.add_plugins(arena);

    loop {
        for bot in &bots {
            let bot_main = bot.instance.exports.get_function("main")?;
            let result = bot_main.call(&mut store, &[])?;
            println!("{:?}", result);
        }
        app.update();
    }
}
