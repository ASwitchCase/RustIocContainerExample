use crate::ioc_application_container::{IocApplicationContainer, Lifetime};

mod ioc_application_container;

fn main() {
    let mut container = IocApplicationContainer::new();

    // 1. Add Singleton: Created once, shared forever
    container.add(Lifetime::Singleton, |_| "Shared Config".to_string());

    // 2. Add Scoped: Created once per "request"
    container.add(Lifetime::Scoped, |_| std::time::Instant::now());

    // --- Simulation: Request 1 ---
    container.begin_scope();
    let s1 = container.resolve::<std::time::Instant>();
    let s2 = container.resolve::<std::time::Instant>();
    assert_eq!(s1, s2); // Same instance within the same scope

    // --- Simulation: Request 2 ---
    container.begin_scope();
    let s3 = container.resolve::<std::time::Instant>();
    assert_ne!(s1, s3); // Different instance in a new scope
}
