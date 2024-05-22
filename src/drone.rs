use crate::messages::{BasicId, Location, SystemMessage, Operator};

#[derive(Debug)]
pub struct Drone {
    pub basic_id: Option<BasicId>,
    pub last_location: Option<Location>,
    pub location_history: Vec<Location>,
    pub system_message: Option<SystemMessage>,
    pub operator: Option<Operator>,
}

impl Drone {
    pub fn new(basic_id: Option<BasicId>, last_location: Option<Location>, system_message: Option<SystemMessage>, operator: Option<Operator>) -> Drone {
        let last_location = last_location.clone();
        let last_location_history = match last_location.clone() {
            Some(location) => vec![location.clone()],
            None => vec![],
        };
        Drone {
            basic_id,
            last_location: last_location.clone(),
            location_history: last_location_history,
            system_message,
            operator,
        }
    }

    pub fn update_basic_id(&mut self, basic_id: BasicId) {
        self.basic_id = Some(basic_id);
    }

    pub fn update_system_message(&mut self, system_message: SystemMessage) {
        self.system_message = Some(system_message);
    }

    pub fn update_operator(&mut self, operator: Operator) {
        self.operator = Some(operator);
    }

    pub fn update_location(&mut self, location: Location) {
        let last_location = self.last_location.clone().unwrap();
        self.location_history.push(last_location.clone());
        self.last_location = Some(location);
    }

    pub fn payload_ready(&self) -> bool {
        self.basic_id.is_some() && self.last_location.is_some() && self.system_message.is_some() && self.operator.is_some()
    }

}
