use bevy::asset::AsyncReadExt;
use bevy::platform::collections::HashMap;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};

use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::*;
use std::sync::Arc;
use thiserror::Error;

pub const GENS: usize = 16;
pub const TIMERS: usize = 4;
pub const TYPES: usize = 8;

#[derive(Debug, Clone)]
// Ген с радиусом распространения морфогена
pub struct Morphogen {
    pub range: isize,
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
pub struct ConditionType {
    v: ConditionValue,
    negate: bool,
}

impl ConditionType {
    pub fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let negate = inner.next().unwrap().as_str() == "!";
        let tag = inner.next().unwrap();

        let v = match tag.as_rule() {
            Rule::mgen => {
                let index: u8 = tag.into_inner().as_str().parse().unwrap();
                ConditionValue::M(index)
            }
            Rule::timer => {
                let index: u8 = tag.into_inner().as_str().parse().unwrap();
                ConditionValue::T(index)
            }
            Rule::is_type => {
                let check: String = tag.into_inner().as_str().into();
                ConditionValue::IsType(check)
            }
            Rule::neighbors => {
                let number = tag.into_inner().as_str().parse().unwrap();
                ConditionValue::Neighbors(number)
            }
            Rule::can_division => ConditionValue::Division,
            _ => unreachable!("Can be reached"),
        };

        Self { v, negate }
    }

    pub fn check(&self, d: bool, n: u8, c: &[u8; GENS], t: &[u8; TIMERS], _type: &String) -> bool {
        let result = match &self.v {
            ConditionValue::M(i) => c[*i as usize] > 0,
            ConditionValue::T(i) => t[*i as usize] > 0,
            ConditionValue::IsType(v) => v == _type,
            ConditionValue::Neighbors(v) => n == *v,
            ConditionValue::Division => d,
        };

        if self.negate {
            return !result;
        }
        return result;
    }
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub activators: Vec<Vec<ConditionType>>,
    pub deactivators: Vec<Vec<ConditionType>>,
}

impl Condition {
    pub fn new() -> Self {
        Self {
            activators: Vec::with_capacity(TYPES),
            deactivators: Vec::with_capacity(TYPES),
        }
    }

    pub fn parse(values: Pairs<Rule>) -> Self {
        let mut result = Self::new();

        for v in values {
            let mut conditions = Vec::new();
            let rule = v.as_rule();

            for inner in v.into_inner() {
                conditions.push(ConditionType::parse(inner));
            }

            match rule {
                Rule::activator => {
                    result.activators.push(conditions);
                }
                Rule::deactivator => {
                    result.deactivators.push(conditions);
                }
                _ => (),
            }
        }

        result
    }

    // Морфогены находится в межклеточном веществе.
    // Любой ген производит морфоген соответствующего типа.
    // Причина активации гена - это существование определенных морфогенов
    // в межклеточном веществе клетки.
    //
    /// d - может ли клетка делится?
    /// n - количество соседей у клетки
    /// c - концентрация генов в веществе клетки
    /// t - гены-таймеры у клетки
    /// _type - Тип текущей клетки
    pub fn check(&self, d: bool, n: u8, c: &[u8; GENS], t: &[u8; TIMERS], _type: &String) -> bool {
        // Если хотя-бы один деактиватор (подавляющий ген) активен, то пропускаем проверки
        for v in &self.deactivators {
            let result = v.iter().all(|cv| cv.check(d, n, c, t, _type));

            if result {
                return false;
            }
        }

        // Если все значения активатора верны, то возвращается true. Иначе проверяем дальше.
        for v in &self.activators {
            let result = v.iter().all(|cv| cv.check(d, n, c, t, _type));

            if result {
                return true;
            }
        }

        // Если никакой активатор не работает, выключаем ген
        return false;
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
                            let range: isize = mgen_inner.into_inner().as_str().parse().unwrap();
                            assert!(range >= 0);

                            let condition = Condition::parse(property);
                            value.gens.insert(index, Morphogen { range, condition });
                        }
                        Rule::timer => {
                            let index: usize = tag.into_inner().as_str().parse().unwrap();
                            let timer_inner = property.next().unwrap();
                            let time: u8 = timer_inner.into_inner().as_str().parse().unwrap();

                            let condition = Condition::parse(property);
                            value.timers.insert(index, Timer { time, condition });
                        }
                        Rule::change => {
                            let mut property_name = property.next().unwrap().into_inner();
                            let name: String = property_name.next().unwrap().as_str().into();

                            let condition = Condition::parse(property);
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
