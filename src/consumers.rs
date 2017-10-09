use std::collections::VecDeque;
use simulators::Packet;

// Consumer has a VecDeque and its buffer size
struct Consumer {
    queue: VecDeque<Packet>,
    buffer_size: usize,
}

impl Consumer {
    fn new(buffer_size: usize) -> Consumer {
        Consumer {
            queue: VecDeque::new(),
            buffer_size: buffer_size,
        }
    }

    // If the consumer doesn't have a buffer size (if it's 0), it enques
    // IF the consumer does have a buffer size, it enques when it's length is less then it's buffer size
    fn enqueue(&mut self, packet: Packet) -> () {
        if self.buffer_size == 0 {
            self.queue.push_back(packet);
        } else if self.queue.len() < self.buffer_size { 
            self.queue.push_back(packet);
        }
    }

    fn dequeue(&mut self) -> Option<Packet> {
        self.queue.pop_front()
    }
}
