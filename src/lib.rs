//! A rust library to communicate with the
//! SOEHNLE Terminal 3005 (via RS232).

#[derive(Debug, Clone)]
pub struct Message {
    pub status: Status,
    pub nr: u8,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct Status {
    pub under_load: bool,
    pub over_load: bool,
    pub standstill: bool,
    pub empty_message: bool,
}
