use std::thread;
use crate::planet::{PatchGeometry, PatchLocation, GeometryProvider};
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver};
use std::sync::mpsc::channel;

static NEXT: AtomicUsize = AtomicUsize::new(0);

pub struct Token {
    pub priority: u32,
}

struct Request {
    id: usize,
    token: Arc<Token>,
    patch_location: PatchLocation,
}

pub trait AsyncGeometryProvider {
    fn queue(&self, patch_location: PatchLocation) -> (Arc<Token>, usize);
    fn receive_all<F: FnMut(usize, PatchGeometry) -> ()>(&self, drain: F);
}


pub struct ThreadpoolGeometryProvider<T: GeometryProvider> {
    provider: Arc<T>,
    queue: Arc<Mutex<Vec<Request>>>,
    is_not_empty: Arc<Condvar>,
    should_stop: Arc<AtomicBool>,
    threads: Vec<thread::JoinHandle<()>>,
    receiver: Receiver<(usize, PatchGeometry)>,
}


impl<T: GeometryProvider + Send + Sync + 'static> ThreadpoolGeometryProvider<T> {
    /// Create new instance
    pub fn new(provider: T) -> ThreadpoolGeometryProvider<T> {
        let (sender, receiver) = channel();

        let mut tgp = ThreadpoolGeometryProvider {
            provider: Arc::new(provider),
            is_not_empty: Arc::new(Condvar::new()),
            should_stop: Arc::new(AtomicBool::new(false)),
            queue: Arc::new(Mutex::new(Vec::new())),
            threads: Vec::new(),
            receiver,
        };

        for _ in 0..3 {
            let thread_provider = tgp.provider.clone();
            let thread_queue = tgp.queue.clone();
            let thread_is_not_empty = tgp.is_not_empty.clone();
            let thread_should_stop = tgp.should_stop.clone();
            let thread_sender = sender.clone();

            let handle = thread::spawn(move || {
                while !thread_should_stop.load(Ordering::Relaxed) {
                    let request = {
                        let mut queue = thread_queue.lock().expect("Could not lock queue");

                        while queue.is_empty() && !thread_should_stop.load(Ordering::Relaxed) {
                            queue = thread_is_not_empty.wait(queue).expect("Could not wait on queue");
                        }

                        if queue.is_empty() {
                            break;
                        }

                        // Sort the queue by priority
                        queue.sort_by(|a, b| {
                            a.token.priority.cmp(&b.token.priority)
                        });

                        queue.pop().expect("Queue is empty, this should be impossible!")
                    };

                    thread_sender.send((request.id, thread_provider.provide(request.patch_location))).expect("Could not send patch result over Channel");
                }
            });
            tgp.threads.push(handle);
        }

        tgp
    }
}


impl<T: GeometryProvider> AsyncGeometryProvider for ThreadpoolGeometryProvider<T> {
    fn queue(&self, patch_location: PatchLocation) -> (Arc<Token>, usize) {
        let next = NEXT.fetch_add(1, Ordering::SeqCst);
        let request = Request { id: next, token: Arc::new(Token { priority: 1 }), patch_location };

        let token = request.token.clone();

        let mut queue = self.queue.lock().expect("Could not lock queue");
        queue.push(request);

        (token, next)
    }

    fn receive_all<F: FnMut(usize, PatchGeometry) -> ()>(&self, mut drain: F) {
        for (id, result) in self.receiver.try_iter() {
            drain(id, result);
        }
    }
}


impl<T: GeometryProvider> GeometryProvider for ThreadpoolGeometryProvider<T> {
    fn provide(&self, patch: PatchLocation) -> PatchGeometry {
        self.provider.provide(patch)
    }
}