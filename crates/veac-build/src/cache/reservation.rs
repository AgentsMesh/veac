use crate::ArtifactOutputs;

pub trait BuildLease: Send {}

impl<T: Send> BuildLease for T {}

pub enum CacheReservation {
    Hit(ArtifactOutputs),
    Owner(Box<dyn BuildLease>),
}
