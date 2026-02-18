use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Factory = Box<dyn Fn(&IocApplicationContainer) -> Arc<dyn Any + Send + Sync> + Send + Sync>;

pub enum Lifetime { Singleton, Scoped, Transient }

pub struct IocApplicationContainer {
    factories: HashMap<TypeId, (Factory, Lifetime)>,
    // Singleton cache: shared across all resolutions
    singletons: Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    // Scoped cache: specific to one "request" or block of work
    scoped: Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl IocApplicationContainer {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            singletons: Mutex::new(HashMap::new()),
            scoped: Mutex::new(HashMap::new()),
        }
    }

    // Helper for registration
    pub fn add<T, F>(&mut self, lifetime: Lifetime, factory: F)
    where T: Send + Sync + 'static, F: Fn(&IocApplicationContainer) -> T + Send + Sync + 'static
    {
        let wrapper = move |c: &IocApplicationContainer| Arc::new(factory(c)) as Arc<dyn Any + Send + Sync>;
        self.factories.insert(TypeId::of::<T>(), (Box::new(wrapper), lifetime));
    }

    pub fn resolve<T>(&self) -> Arc<T> 
    where T: Send + Sync + 'static
    {
        let tid = TypeId::of::<T>();
        let (factory, lifetime) = self.factories.get(&tid).expect("Not registered");

        match lifetime {
            Lifetime::Transient => factory(self).downcast::<T>().unwrap(),
            Lifetime::Singleton => {
                let mut cache = self.singletons.lock().unwrap();
                cache.entry(tid).or_insert_with(|| factory(self))
                    .clone().downcast::<T>().unwrap()
            },
            Lifetime::Scoped => {
                let mut cache = self.scoped.lock().unwrap();
                cache.entry(tid).or_insert_with(|| factory(self))
                    .clone().downcast::<T>().unwrap()
            }
        }
    }

    // Call this at the start of a new request to clear "Scoped" services
    pub fn begin_scope(&self) {
        self.scoped.lock().unwrap().clear();
    }
}
