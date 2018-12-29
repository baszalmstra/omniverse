use std::thread;
use crate::planet::{PatchGeometry, PatchLocation, GeometryProvider};
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

static NEXT: AtomicUsize = AtomicUsize::new(0);

pub struct Token {
    pub priority: AtomicUsize,
}

struct Request {
    id: usize,
    token: Arc<Token>,
    patch_location: PatchLocation,
}

pub trait AsyncGeometryProvider {
    /// Queues the patch location for processing at a later time, returns a token with a priority
    /// and an id to identify the patch later
    fn queue(&self, patch_location: PatchLocation) -> (Arc<Token>, usize);

    /// Receives all values that have been processed and passes these to a callback function
    /// that can use them as it pleases
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

    /// Create new instance, all threads are started and waiting for patches to be generated
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

                        // If this is the case, then we should stop
                        if queue.is_empty() {
                            break;
                        }

                        // Drop all cancelled tokens
                        queue.retain(|a| a.token.priority.load(Ordering::SeqCst) != 0);
                        // Sort the queue by priority
                        queue.sort_by(|a, b| {
                            a.token.priority.load(Ordering::SeqCst).cmp(&b.token.priority.load(Ordering::SeqCst))
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


/// Implementation of the async code for the Geometry provider working with a threadpool
impl<T: GeometryProvider> AsyncGeometryProvider for ThreadpoolGeometryProvider<T> {
    /// Queue and process the patches asynchronously
    fn queue(&self, patch_location: PatchLocation) -> (Arc<Token>, usize) {
        let next = NEXT.fetch_add(1, Ordering::SeqCst);
        let request = Request { id: next, token: Arc::new(Token { priority: AtomicUsize::new(1) }), patch_location };

        let token = request.token.clone();

        let mut queue = self.queue.lock().expect("Could not lock queue");
        queue.push(request);

        self.is_not_empty.notify_one();

        (token, next)
    }

    /// Receive all values sent over the channel
    fn receive_all<F: FnMut(usize, PatchGeometry) -> ()>(&self, mut drain: F) {
        for (id, result) in self.receiver.try_iter() {
            drain(id, result);
        }
    }
}

/// Implement drop for provider so threads are stopped
impl<T: GeometryProvider> Drop for ThreadpoolGeometryProvider<T>{
    fn drop(&mut self) {
        {
            let mut queue = self.queue.lock().expect("Could not lock queue to drop value");
            queue.clear()
        }
        self.should_stop.store(true, Ordering::SeqCst);
        self.is_not_empty.notify_all();
    }
}


/// This is a geometry provider that acts like a async provider but is
/// actually an synchronous provider
pub struct SyncGeometryProvider<T: GeometryProvider> {
    provider: T,
    sender: Sender<(usize, PatchGeometry)>,
    receiver: Receiver<(usize, PatchGeometry)>,
}

impl<T: GeometryProvider> SyncGeometryProvider<T> {
    /// Create new sync geometry provider
    pub fn new(provider: T) -> SyncGeometryProvider<T> {
        let (sender, receiver) = channel();

        SyncGeometryProvider {
            provider,
            sender,
            receiver,
        }
    }
}

impl<T: GeometryProvider> AsyncGeometryProvider for SyncGeometryProvider<T> {
    /// Queue and process directly, send directly over a channel
    fn queue(&self, patch_location: PatchLocation) -> (Arc<Token>, usize) {
        let next = NEXT.fetch_add(1, Ordering::SeqCst);
        let token = Arc::new(Token { priority: AtomicUsize::new(1) });
        self.sender.send((next, self.provider.provide(patch_location))).expect("Could not send processing result over channel");
        (token, next)
    }

    /// Receive all values sent over channel
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

impl<T: GeometryProvider> GeometryProvider for SyncGeometryProvider<T> {
    fn provide(&self, patch: PatchLocation) -> PatchGeometry {
        self.provider.provide(patch)
    }
}