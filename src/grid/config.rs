use bevy::asset::AsyncReadExt;
use bevy::platform::collections::HashMap;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};

use pest::Parser;
use pest::iterators::Pairs;
use pest_derive::*;
use std::sync::Arc;
use thiserror::Error;

pub const GENS: usize = 16;
pub const TIMERS: usize = 4;
pub const TYPES: usize = 8;

#[derive(Debug, Clone)]
// Ген с радиусом распространения морфогена
pub struct Morphogen {
    pub range: u8,
    pub condition: Condition,
}

#[derive(Debug, Clone)]
// Ген с установленным таймером
pub struct Timer {
    pub time: u8,
    pub condition: Condition,
}

// Ген который отвечает за дифференцировку клетки
#[derive(Debug, Clone)]
pub struct Change(String);

#[derive(Debug, Clone)]
pub enum ConditionValue {
    M(u8),
    T(u8),
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
    activators: Vec<ConditionType>,
    deactivators: Vec<ConditionType>,
}

impl Condition {
    pub fn new() -> Self {
        Self {
            activators: Vec::with_capacity(TYPES),
            deactivators: Vec::with_capacity(TYPES),
        }
    }

    pub fn parse(values: Pairs<Rule>) -> Self {
        let result = Self::new();

        for value in values {
            match value.as_rule() {
                Rule::activator => {}
                Rule::deactivator => {}
                _ => (),
            }
        }

        result
    }
}

#[derive(Default, Debug, Clone)]
/// Тип клетки определяет её поведение, клетка ссылается к типу
/// и определяет следующий тик в зависимости от него
pub struct CellType {
    pub name: String,
    /// Условия работы генов
    pub gens: HashMap<usize, Morphogen>,
    /// Условия запуска таймеров
    pub timers: HashMap<usize, Timer>,
    /// Условия дифференцировки клетки на другой тип
    pub changes: HashMap<String, Condition>,
    // Условия начала деления клетки
    pub division: Option<Condition>,
    // Цвет клетки
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
    pub types: HashMap<String, Arc<CellType>>,
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
                        config.types.insert(value.name.clone(), Arc::new(value));
                    } else {
                        config.default = name.to_string();
                    }

                    cell = Some(CellType::new(name));
                }
                Rule::property => {
                    if cell.is_none() {
                        continue;
                    }
                    let mut value = cell.take().unwrap();
                    let mut property = inner.into_inner();

                    let tag = property.next().unwrap();
                    match tag.as_rule() {
                        Rule::mgen => {
                            let index: usize = tag.into_inner().as_str().parse().unwrap();
                            let mgen_inner = property.next().unwrap();
                            let range: u8 = mgen_inner.into_inner().as_str().parse().unwrap();

                            let condition = Condition::parse(property.next().unwrap().into_inner());
                            value.gens.insert(index, Morphogen { range, condition });
                        }
                        Rule::timer => {
                            let index: usize = tag.into_inner().as_str().parse().unwrap();
                            let timer_inner = property.next().unwrap();
                            let time: u8 = timer_inner.into_inner().as_str().parse().unwrap();

                            let condition = Condition::parse(property.next().unwrap().into_inner());
                            value.timers.insert(index, Timer { time, condition });
                        }
                        Rule::change => {
                            let mut property_name = property.next().unwrap().into_inner();
                            let name: String = property_name.next().unwrap().as_str().into();

                            let condition = Condition::parse(property.next().unwrap().into_inner());
                            value.changes.insert(name, condition);
                        }
                        _ => (),
                    };

                    cell = Some(value);
                }
                Rule::color => {
                    if cell.is_none() {
                        continue;
                    }

                    let mut value = cell.take().unwrap();
                    value.color = Srgba::hex(inner.into_inner().as_str()).unwrap_or(Srgba::BLACK);
                    cell = Some(value);
                }
                Rule::division => {
                    if cell.is_none() {
                        continue;
                    }

                    let mut value = cell.take().unwrap();
                    value.division = Some(Condition::parse(inner.into_inner()));
                    cell = Some(value);
                }
                _ => (),
            }
        }

        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &["sim"]
    }
}
