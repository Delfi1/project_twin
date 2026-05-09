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
use thiserror::Error;

pub const ACTIVATORS: usize = 8;
pub const GENS: usize = 16;
pub const TIMERS: usize = 4;

#[derive(Debug, Default, Clone, Copy)]
pub struct Conditions {}

#[derive(Debug, Default, Clone, Copy)]
pub enum ValueType {
    #[default]
    None,
    // У гена есть радиус воспроизводства - количество клеток
    Gen(u8),
    // Таймер работает N тиков и пока он работает, его можно использовать в условиях выработки морфогенов
    Timer(u8),
    // Этот тип определяет может ли клетка делится
    Division,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Value {
    t: ValueType,
    // Условия при которых морфоген производится
    activators: [Conditions; ACTIVATORS],
    // Условия при которых морфоген НЕ производится
    deactivators: [Conditions; ACTIVATORS],
}

#[derive(Default, Debug, Clone, Copy)]
pub struct CellType {
    pub gens: [Value; GENS],
    pub timers: [Value; TIMERS],
    pub division: Value,

    pub color: Srgba,
}

#[derive(Asset, Clone, Default, TypePath, Debug)]
pub struct Parser {
    // Тип клетки "по умолчанию"
    pub default: String,
    pub types: HashMap<String, CellType>,
}

#[derive(Default, TypePath)]
pub struct ParserLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ParserLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for ParserLoader {
    type Asset = Parser;
    type Settings = ();
    type Error = ParserLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("Loading parser asset...");
        let mut _parser = Parser::default();
        let mut bytes = String::with_capacity(1024);
        reader.read_to_string(&mut bytes).await?;

        _parser.default = "stem".to_string();
        let mut cell = CellType::default();
        cell.color = Srgba::BLACK;
        _parser.types.insert(_parser.default.clone(), cell);

        Ok(_parser)
    }

    fn extensions(&self) -> &[&str] {
        &["sim"]
    }
}
