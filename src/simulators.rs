use std::collections::VecDeque;
use generators::Generator;

// Packet holds the value of the time unit that it was generated at.
pub struct Packet(pub u32);

// ClientStatistics is the set of statistics we care about post-simulation as far as the client is
// concerned.
pub struct ClientStatistics {
    pub packets_generated: u32,
}

impl ClientStatistics {
    fn new() -> ClientStatistics {
        ClientStatistics { packets_generated: 0 }
    }
}

// Client generates packets according as per the parametrized generators::Generator. We maintain a
// ticker count to the next time a packet is to be generated, moving forward at ticks of the
// specified resolution. We also collect client statistics through this progression.
pub struct Client<G: Generator> {
    resolution: f64,
    ticker: u32,
    generator: G,
    pub statistics: ClientStatistics,
}

impl<G: Generator> Client<G> {
    // Client::new seeds the ticker using the provided generator.
    pub fn new(generator: G, resolution: f64) -> Client<G> {
        Client {
            ticker: generator.next_event(resolution),
            generator: generator,
            statistics: ClientStatistics::new(),
            resolution: resolution,
        }
    }

    // The caller is responsible for calling Client.tick() at fixed time intervals, moving the
    // client simulator one time unit per call. We return a boolean indicating whether or not a
    // packet is generated in the most recently completed time unit.
    //
    // We're careful to check if self.ticker == 0 before decrementing because the parametrized
    // generator may very well return 0 (see top-level comment in src/generators.rs).
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

// ServerStatistics is the set of statistics we care about post-simulation as far as the server is
// concerned.
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

// Server stores packets in a queue and processes them. See comment on struct Client for the used
// ticker pattern. Similarly we maintain server statistics as the server moves through the
// simulation.
pub struct Server<G: Generator> {
    queue: VecDeque<Packet>,
    buffer_limit: Option<usize>,
    ticker: u32,
    resolution: f64,
    generator: G,
    pub statistics: ServerStatistics,
}

impl<G: Generator> Server<G> {
    // Server::new returns a server with the specified buffer limit, if any. We also set seed the
    // ticker using the provided generator as done so in Client::new.
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

    // Server.enqueue enqueues a packet for delivery. If the packet is to be dropped (due to the
    // internal queue being full it is recorded in the server's internal statistics.
    pub fn enqueue(&mut self, packet: Packet) {
        match self.buffer_limit {
            Some(limit) => {
                if self.queue.len() < limit {
                    self.queue.push_back(packet);
                } else {
                    self.statistics.packets_dropped += 1
                }
            }
            // Infinite queue, limit == None.
            None => {
                self.queue.push_back(packet);
            }
        }
    }

    // See comment above Client.tick for the use of the tick pattern. The return value here, if not
    // None, is the packet that has finished being processed.
    //
    // The server operates via polling essentially, we process an entire packet each time the
    // ticker returns to zero, if any. If the queue is empty this means the server is idle for that
    // time duration and is recorded so internally.
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

    // Server.qlen returns the number of packets in the server's internal buffer, waiting to be
    // processed.
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

    #[test]
    fn server_idle_count() {
        let mut s = Server::new(Deterministic::new(1.0), 1.0, Some(1));

        s.tick();
        assert_eq!(s.statistics.idle_count, 1);

        s.tick();
        assert_eq!(s.statistics.idle_count, 2);

        s.enqueue(Packet(0));
        s.tick();
        assert_eq!(s.statistics.idle_count, 2);
        assert_eq!(s.statistics.packets_processed, 1);
    }
}
