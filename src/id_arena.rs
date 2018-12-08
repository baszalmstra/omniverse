use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

pub trait IdGenerator {
    type Id;

    /// Acquires a new id
    fn acquire(&mut self) -> Option<Self::Id>;

    /// Releases the specified Id
    fn release(&mut self, id: Self::Id);

    /// Returns the number of Ids allocated
    fn len(&self) -> usize;
}

pub trait IdArena: IdGenerator {
    /// Returns the maximum capacity of the arena
    fn capacity(&self) -> usize;
}

pub struct SimpleIdArena {
    counter: AtomicUsize,
    free: Mutex<Vec<usize>>,
    capacity: usize,
}

impl SimpleIdArena {
    pub fn with_capacity(capacity: usize) -> SimpleIdArena {
        SimpleIdArena {
            counter: AtomicUsize::new(0),
            free: Mutex::new(Vec::new()),
            capacity,
        }
    }
}

impl IdGenerator for SimpleIdArena {
    type Id = usize;

    fn acquire(&mut self) -> Option<Self::Id> {
        self.free
            .try_lock()
            .ok()
            .and_then(|mut free| free.pop())
            .or_else(|| {
                let id = self.counter.fetch_add(1, Ordering::Relaxed);
                if id >= self.capacity {
                    self.counter.fetch_sub(1, Ordering::Relaxed);
                    None
                } else {
                    Some(id)
                }
            })
    }

    fn release(&mut self, id: Self::Id) {
        self.free.lock().unwrap().push(id);
    }

    fn len(&self) -> usize {
        self.counter.load(Ordering::Relaxed) - self.free
            .try_lock().unwrap().len()
    }
}

impl IdArena for SimpleIdArena {
    fn capacity(&self) -> usize {
        self.capacity
    }
}
