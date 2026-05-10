use bevy::asset::AsyncReadExt;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};

use pest::Parser;
use pest_derive::*;
use thiserror::Error;

pub const GENS: usize = 16;
pub const TIMERS: usize = 4;
pub const CONDITIONS: usize = 8;
pub const TYPES: usize = 8;

pub enum ConditionValue {
    // Morphogen and timer id
    Morphogen(u8),
    Timer(u8),

    // Count of neighbors
    Neighbors(u8),
    // Cell type
    Type(&'static str),
}

// Значение которое присваивается клетке
#[derive(Default, Debug, Clone, Copy)]
pub struct Condition {
    activators: [&'static str; CONDITIONS],
    deactivators: [&'static str; CONDITIONS],
}

#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    // Расстояние работы морфогена
    Morphogen(u8),
    // Время отсчёта таймера
    Timer(u8),
    Change(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub struct Value {
    _type: ValueType,
    condition: Condition,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct CellType {
    pub gens: [Option<Value>; GENS],
    pub timers: [Option<Value>; TIMERS],
    pub changes: [Option<Value>; TYPES],

    pub division: Condition,
    // gens, timers, color...
    pub color: Srgba,
}

#[derive(Parser)]
#[grammar = "assets/parser.pest"]
// Парсер файла конфигурации симуляции
pub struct ConfigParser;

#[derive(Asset, Clone, Default, TypePath, Debug)]
pub struct Config {
    // Тип клетки "по умолчанию"
    pub default: String,
    pub types: Vec<CellType>,
}

#[derive(Default, TypePath)]
pub struct ConfigLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ConfigLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Config parse error: {0}")]
    Parse(#[from] pest::error::Error<Rule>),
}

impl AssetLoader for ConfigLoader {
    type Asset = Config;
    type Settings = ();
    type Error = ConfigLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("Loading simulation config...");
        let mut data = String::with_capacity(1024);
        reader.read_to_string(&mut data).await?;

        let mut config = Config::default();

        // Todo: remove default cell type
        let mut cell = CellType::default();
        config.default = "Stem".to_string();
        cell.color = Srgba::BLACK;
        config.types.push(cell);

        for pair in ConfigParser::parse(Rule::config, &data)? {
            let mut _cell = CellType::default();

            println!("Rule: {:?}", pair.as_rule());
            println!("Span: {:?}", pair.as_span());
        }

        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &["sim"]
    }
}
