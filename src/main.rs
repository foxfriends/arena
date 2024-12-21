use bevy::prelude::*;
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;
use wasmer::{imports, Imports, Instance, Module, Store, Value};

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

    fn register_bot(app: &mut App, bot: Bot);
}

struct Digger;

impl Arena for Digger {
    fn register_bot(_app: &mut App, _bot: Bot) {}
}

impl Plugin for Digger {
    fn build(&self, _app: &mut App) {}
}

struct Bot {
    name: String,
    instance: Instance,
}

#[derive(Resource)]
struct Actions(HashMap<String, String>);

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
        });
    }

    let mut app = App::new();
    app.add_plugins(arena);

    for bot in &mut bots {
        let init = bot.instance.exports.get_function("init")?;
        init.call(&mut store, &[])?;
    }

    loop {
        let mut actions = HashMap::new();
        for bot in &mut bots {
            let step = bot.instance.exports.get_function("step")?;
            let result = step.call(&mut store, &[])?;
            let offset = result[0].unwrap_i64() as u64;
            let length = result[1].unwrap_i64() as usize;

            let arena_memory = bot.instance.exports.get_memory("arena")?;
            let view = arena_memory.view(&store);
            let mut buffer = vec![0; length as usize];
            view.read(offset, &mut buffer)?;
            actions.insert(bot.name.clone(), String::from_utf8(buffer)?);
        }
        app.insert_resource(Actions(actions)).update();
    }
}
