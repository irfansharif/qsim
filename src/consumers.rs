use std::collections::VecDeque;
use simulators::Packet;

// Consumer stores packets in a queue and processes them
#[allow(dead_code)]
pub struct Consumer {
    queue: VecDeque<Packet>,
    buffer_limit: Option<usize>,
    service_time: u32,
    state: State,
}

#[derive(Debug, PartialEq)]
pub enum State {
    #[allow(dead_code)]
    Idle,
    // Processing holds the time that the consumer will be in processing till
    #[allow(dead_code)]
    Processing(u32),
}

impl Consumer {
    #[allow(dead_code)]
    fn new(buffer_limit: Option<usize>, service_time: u32) -> Consumer {
        Consumer {
            queue: VecDeque::new(),
            buffer_limit: buffer_limit,
            service_time: service_time,
            state: State::Idle,
        }
    }

    #[allow(dead_code)]
    fn enqueue(&mut self, packet: Packet) -> bool {
        match self.buffer_limit {
            Some(size) => {
                if self.queue.len() < size {
                    self.queue.push_back(packet);
                    true
                } else {
                    false
                }
            }
            // buffer_size of none implies that it is an infinite queue
            None => {
                self.queue.push_back(packet);
                true
            }
        }
    }

    #[allow(dead_code)]
    fn consume(&mut self, time: u32) -> Option<Packet> {
        match self.state {
            State::Idle => {
                if !self.queue.is_empty() {
                    self.state = State::Processing(time + self.service_time);
                }
                None
            }
            State::Processing(dt) => {
                let packet = if time >= dt {
                    self.queue.pop_front()
                } else {
                    None
                };
                if self.queue.is_empty() {
                    self.state = State::Idle;
                } else {
                    self.state = State::Processing(time + self.service_time);
                }
                packet
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume() {
        let mut consumer = Consumer::new(None, 1);
        consumer.queue.push_back(Packet(1));
        consumer.queue.push_back(Packet(2));
        let packet = consumer.consume(1);
        assert_eq!(consumer.state, State::Processing(2));
        assert!(packet.is_none());
        let packet = consumer.consume(2);
        assert_eq!(consumer.state, State::Processing(3));
        assert_eq!(packet.expect("Should be 1").0, 1);
        consumer.consume(3);
        assert_eq!(consumer.state, State::Idle);
    }
}
