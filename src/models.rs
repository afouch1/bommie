
#[derive(Default)]
pub struct Unit {
    pub name: String,
    pub quantity: u32
}

impl Clone for Unit {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            quantity: self.quantity
        }
    }
}

#[derive(Default)]
pub struct Print {
    pub name: String,
    pub units: Vec<Unit>,
    pub potential_unit: Unit
}

#[derive(Default)]
pub struct BommieApp {
    pub current_print: usize,
    pub prints: Vec<Print>,
    pub potential_print: String,
    pub error_message: Option<String>
}
