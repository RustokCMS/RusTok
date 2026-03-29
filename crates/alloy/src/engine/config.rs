use std::time::Duration;

#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Максимум операций на один запуск
    pub max_operations: u64,

    /// Таймаут выполнения
    pub timeout: Duration,

    /// Максимум глубины вызова функций
    pub max_call_depth: usize,

    /// Максимум размера строки (bytes)
    pub max_string_size: usize,

    /// Максимум размера массива
    pub max_array_size: usize,

    /// Максимум глубины вложенных объектов
    pub max_map_depth: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_operations: 50_000,
            timeout: Duration::from_millis(100),
            max_call_depth: 16,
            max_string_size: 64 * 1024,
            max_array_size: 10_000,
            max_map_depth: 16,
        }
    }
}

impl EngineConfig {
    pub fn relaxed() -> Self {
        Self {
            max_operations: 500_000,
            timeout: Duration::from_secs(5),
            ..Default::default()
        }
    }

    pub fn strict() -> Self {
        Self {
            max_operations: 10_000,
            timeout: Duration::from_millis(50),
            max_call_depth: 8,
            ..Default::default()
        }
    }
}
