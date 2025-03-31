/// Wrapper function for generating a new Uuid. If the "uuid-disable-random" feature is enabled, then will return nil - otherwise a random Uuid is generated and returned.
pub fn generate_uuid() -> uuid::Uuid {
    #[cfg(not(feature = "uuid-disable-random"))]
    {
        uuid::Uuid::new_v4()
    }

    #[cfg(feature = "uuid-disable-random")]
    {
        uuid::Uuid::nil()
    }
}