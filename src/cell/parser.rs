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

use bevy::platform::collections::HashMap;
use bevy::prelude::*;

/// Парсер будет читать эти значения из файла
#[derive(Debug, Default, Clone, Copy)]
pub enum Value {
    #[default]
    None,
    Gen {
        radius: u8,
    },
    Timer {
        ticks: u8,
    },
}

pub const GENS: usize = 16;
pub const TIMERS: usize = 4;

#[derive(Debug, Clone, Copy)]
pub struct CellType {
    gens: [Value; GENS],
    timers: [Value; TIMERS],
}

#[derive(Asset, Default, TypePath, Debug)]
pub struct Parser {
    default: String,
    types: HashMap<String, CellType>,
}

#[derive(Resource)]
pub struct WorldParser(pub Handle<Parser>);
