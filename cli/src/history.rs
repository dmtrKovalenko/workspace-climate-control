#![allow(dead_code)]
use crate::climate_data::ClimateData;

pub struct MaxSizedVector<T, const MAX_SIZE: usize> {
    data: Vec<T>,
}

impl<T, const MAX_SIZE: usize> MaxSizedVector<T, MAX_SIZE> {
    /// Constructs a new MaxSizedVector with a specified maximum size
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(MAX_SIZE),
        }
    }

    /// Adds an element to the vector, removing the oldest if at max capacity
    pub fn push(&mut self, item: T) {
        if self.data.len() == MAX_SIZE {
            self.data.remove(0);
        }
        self.data.push(item);
    }

    /// Returns a reference to the element at the given index if it exists
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    /// Checks if the vector is at its maximum capacity
    pub fn is_full(&self) -> bool {
        self.data.len() == MAX_SIZE
    }
    /// Returns the number of elements currently in the vector
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Removes all elements from the vector
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Provides an iterator over the elements of the vector
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    /// Extracts a slice containing the entire vector.
    pub fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }
}

// amount of 5 seconds intervals in 24 hours
const HISTORY_SIZE: usize = 17280;

/// .0 - timestamp, .1 - value
type HistoryPoint = (f64, f64);

pub struct History {
    time_window: [f64; 2],
    pub flat: MaxSizedVector<ClimateData, HISTORY_SIZE>,
    pub co2_history: MaxSizedVector<HistoryPoint, HISTORY_SIZE>,
    pub eco2_history: MaxSizedVector<HistoryPoint, HISTORY_SIZE>,
    pub temperature_history: MaxSizedVector<HistoryPoint, HISTORY_SIZE>,
    pub pressure_history: MaxSizedVector<HistoryPoint, HISTORY_SIZE>,
}

impl History {
    pub fn get_window(&self) -> [f64; 2] {
        self.time_window
    }
    pub fn new() -> Self {
        let now = chrono::offset::Local::now().timestamp_millis() as f64;
        Self {
            time_window: [now, now],
            flat: MaxSizedVector::new(),
            co2_history: MaxSizedVector::new(),
            eco2_history: MaxSizedVector::new(),
            temperature_history: MaxSizedVector::new(),
            pressure_history: MaxSizedVector::new(),
        }
    }

    pub fn capture_measurement(&mut self, climate_data: &ClimateData) {
        let now = chrono::offset::Local::now().timestamp_millis() as f64;
        self.time_window[1] = now;

        self.flat.push(*climate_data);
        self.temperature_history
            .push((now, climate_data.temperature as f64));

        if let Some(co2) = climate_data.co2 {
            self.co2_history.push((now, co2 as f64));
        }

        self.eco2_history.push((now, climate_data.eco2 as f64));
        self.pressure_history
            .push((now, climate_data.pressure as f64));
    }
}
