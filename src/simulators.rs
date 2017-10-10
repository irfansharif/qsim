use std::collections::VecDeque;
use generators::Generator;

// Packet holds the value of the time in ticks that it was generated at.
#[derive(Debug)]
pub struct Packet(pub u32);

#[derive(Debug)]
pub struct ClientStatistics {
    pub packets_generated: u32,
}

impl ClientStatistics {
    fn new() -> ClientStatistics {
        ClientStatistics { packets_generated: 0 }
    }
}
// Client generates packets according as per the parametrized generators::Generator.
#[allow(dead_code)]
pub struct Client<G: Generator> {
    // TODO(irfansharif): Update comments.
    resolution: f64,
    ticker: u32,
    generator: G,
    pub statistics: ClientStatistics,
}

// TODO(irfansharif): Add comments.
impl<G: Generator> Client<G> {
    #[allow(dead_code)]
    pub fn new(generator: G, resolution: f64) -> Client<G> {
        Client {
            ticker: generator.next_event(resolution),
            generator: generator,
            statistics: ClientStatistics::new(),
            resolution: resolution,
        }
    }

    #[allow(dead_code)]
    pub fn tick(&mut self) -> bool {
        // TODO(irfansharif): Resolution mismatch; no possibility of generating multiple packets.
        if self.ticker == 0 {
            self.statistics.packets_generated += 1;
            self.ticker = self.generator.next_event(self.resolution);
            return true;
        }

        self.ticker -= 1;
        if self.ticker == 0 {
            self.statistics.packets_generated += 1;
            self.ticker = self.generator.next_event(self.resolution);
            true
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct ServerStatistics {
    pub packets_processed: u32,
    pub packets_dropped: u32,
    pub idle_count: u32,
}

impl ServerStatistics {
    fn new() -> ServerStatistics {
        ServerStatistics {
            packets_processed: 0,
            packets_dropped: 0,
            idle_count: 0,
        }
    }
}

// Server stores packets in a queue and processes them.
#[allow(dead_code)]
pub struct Server<G: Generator> {
    queue: VecDeque<Packet>,
    buffer_limit: Option<usize>,
    // TODO(irfansharif): Comment.
    ticker: u32,
    resolution: f64,
    generator: G,
    pub statistics: ServerStatistics,
}

impl<G: Generator> Server<G> {
    #[allow(dead_code)]
    pub fn new(generator: G, resolution: f64, buffer_limit: Option<usize>) -> Server<G> {
        Server {
            queue: VecDeque::new(),
            buffer_limit: buffer_limit,
            resolution: resolution,
            ticker: generator.next_event(resolution),
            statistics: ServerStatistics::new(),
            generator: generator,
        }
    }

    #[allow(dead_code)]
    pub fn enqueue(&mut self, packet: Packet) {
        match self.buffer_limit {
            Some(limit) => {
                if self.queue.len() < limit {
                    self.queue.push_back(packet);
                } else {
                    self.statistics.packets_dropped += 1
                }
            }
            // Infinite queue.
            None => {
                self.queue.push_back(packet);
            }
        }
    }

    // TODO(irfansharif): Comment.
    #[allow(dead_code)]
    pub fn tick(&mut self) -> Option<Packet> {
        if self.ticker == 0 {
            self.ticker = self.generator.next_event(self.resolution);
            let packet = self.queue.pop_front();
            match packet {
                Some(_) => self.statistics.packets_processed += 1,
                None => self.statistics.idle_count += 1,
            }
            return packet;
        }

        self.ticker -= 1;
        if self.ticker == 0 {
            self.ticker = self.generator.next_event(self.resolution);
            let packet = self.queue.pop_front();
            match packet {
                Some(_) => self.statistics.packets_processed += 1,
                None => self.statistics.idle_count += 1,
            }
            packet
        } else {
            None
        }
    }

    pub fn qlen(&self) -> usize {
        self.queue.len()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::generators::Deterministic;

    #[test]
    fn client_packet_generation() {
        let mut c = Client::new(Deterministic::new(0.5), 1.0);
        assert!(!c.tick());
        assert!(c.tick());
    }

    #[test]
    fn server_packet_delivery() {
        let mut s = Server::new(Deterministic::new(0.5), 1.0, None);
        s.enqueue(Packet(0));

        s.tick();
        assert_eq!(s.statistics.packets_processed, 0);

        s.tick();
        assert_eq!(s.statistics.packets_processed, 1);

        s.tick();
        assert_eq!(s.statistics.packets_processed, 1);

        s.tick();
        assert_eq!(s.statistics.packets_processed, 1);
    }

    #[test]
    fn server_packet_dropped() {
        let mut s = Server::new(Deterministic::new(1.0), 1.0, Some(1));
        s.enqueue(Packet(0));
        s.enqueue(Packet(0));

        s.tick();
        assert_eq!(s.statistics.packets_processed, 1);
        assert_eq!(s.statistics.packets_dropped, 1);
    }
}
