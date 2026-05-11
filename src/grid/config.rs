use bevy::asset::AsyncReadExt;
use bevy::platform::collections::HashMap;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};

use pest::Parser;
use pest_derive::*;
use thiserror::Error;

pub const GENS: usize = 16;
pub const TIMERS: usize = 4;
pub const TYPES: usize = 8;

#[derive(Debug, Clone, Copy)]
// Ген с радиусом распространения морфогена
pub struct Morphogen(u8);

#[derive(Debug, Clone, Copy)]
// Ген с установленным таймером
pub struct Timer(u8);

// Ген который отвечает за дифференцировку клетки
#[derive(Debug, Clone)]
pub struct Change(String);

#[derive(Debug, Clone)]
pub enum ConditionValue {
    IsType(String),
    Neighbors(u8),
    Division,
}

#[derive(Debug, Clone)]
pub enum ConditionType {
    Default(ConditionValue),
    Negative(ConditionValue),
}

#[derive(Debug, Clone)]
pub struct Condition {
    activators: [ConditionType; 32],
    deactivators: [ConditionType; 32],
}

#[derive(Default, Debug, Clone)]
pub struct CellType {
    pub name: String,
    pub gens: [Option<Morphogen>; GENS],
    pub timers: HashMap<usize, Condition>,
    // Условия дифференцировки клетки на другой тип
    pub changes: HashMap<String, Condition>,

    // gens, timers, color...
    pub color: Srgba,
}

impl CellType {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: Srgba::BLACK,
            ..Default::default()
        }
    }
}

#[derive(Parser)]
#[grammar = "assets/parser.pest"]
// Парсер файла конфигурации симуляции
pub struct ConfigParser;

#[derive(Asset, Clone, Default, TypePath, Debug)]
pub struct Config {
    // Тип клетки "по умолчанию"
    pub default: String,
    pub types: HashMap<String, CellType>,
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

        // Todo: remove default cell type
        let file = ConfigParser::parse(Rule::file, &data)?.next().unwrap();

        let mut config = Config::default();

        let mut cell = None;
        for inner in file.into_inner() {
            match inner.as_rule() {
                Rule::section => {
                    let name = inner.into_inner().next().unwrap().as_str();

                    if cell.is_some() {
                        let value: CellType = cell.take().unwrap();
                        config.types.insert(value.name.clone(), value);
                    } else {
                        config.default = name.to_string();
                    }

                    cell = Some(CellType::new(name));
                }
                Rule::property => {
                    if cell.is_none() {
                        continue;
                    }
                    let mut _value = cell.take().unwrap();

                    //println!("Cell {}: {:?}", value.name, inner.into_inner());

                    cell = Some(_value);
                }
                Rule::color => {
                    if cell.is_none() {
                        continue;
                    }

                    let mut value = cell.take().unwrap();
                    value.color = Srgba::hex(inner.into_inner().as_str()).unwrap_or(Srgba::BLACK);
                    cell = Some(value);
                }
                Rule::division => {}
                _ => (),
            }
        }

        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &["sim"]
    }
}
