//! Парсер генетической информации из файла gen.sim
// Сигнатура файла:
// --------------------------------------
// Тип:
//     <Ген>: [Условие активации](Радиус) {Отключаемые морфогены}
// --------------------------------------
// Пример:
// stem:
//     M0: M0
//     T0: M0 M1 M2 | M3 M4
// --------------------------------------
// Индекс - первое число гена
//
// Типы генов:
// M(index) <- Морфоген
// T(index) <- Таймер. Ожидает N тиков перед активацией, количество тиков задаётся в активирующем гене отдельно

use bevy::asset::AsyncReadExt;
use bevy::platform::collections::HashMap;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};

use pest::Parser;
use pest_derive::*;
use thiserror::Error;

pub const CONDITIONS: usize = 8;
pub const GENS: usize = 16;
pub const TIMERS: usize = 4;

// Index of condion
pub enum ConditionType {
    Morphogen(u8),
    Timer(u8),
    Division(u8),
    Type(&'static str),
}

#[derive(Default, Debug, Clone, Copy)]
pub struct CellType {
    // gens, timers, color...

    // Hex color
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
}

enum ConfigValue<'a> {
    Activators(Vec<&'a str>),
    Deactivators(Vec<&'a str>),
    Color(&'a str),
    Null,
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
        //let nodes = ConfigParser::parse(Rule::ident_list, &data);

        config.default = "stem".to_string();
        let mut cell = CellType::default();
        cell.color = Srgba::BLACK;
        config.types.insert(config.default.clone(), cell);

        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &["sim"]
    }
}
