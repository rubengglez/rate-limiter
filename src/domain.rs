pub trait Limiter {
    fn use_token(&mut self, ip: String) -> Result<(), String>;
}

pub trait Stoppable {
    fn stop(&self);
}
