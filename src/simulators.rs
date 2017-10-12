use std::collections::VecDeque;
use generators::Generator;

// Packet holds the value of the time unit that it was generated at, and its length.
#[derive(Clone)]
pub struct Packet {
    pub time_generated: u32,
    pub length: u32,
}

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

// Server stores packets in a queue and processes them.
pub struct Server {
    queue: VecDeque<Packet>,
    buffer_limit: Option<usize>,
    resolution: f64,
    pub statistics: ServerStatistics,
    // Processing variables
    pspeed: f64,
    curr_packet: Option<Packet>,
    bits_processed: f64,
}

impl Server {
    // Server::new returns a server with the specified buffer limit, if any.
    pub fn new(resolution: f64, pspeed: f64, buffer_limit: Option<usize>) -> Server {
        Server {
            queue: VecDeque::new(),
            buffer_limit: buffer_limit,
            resolution: resolution,
            statistics: ServerStatistics::new(),
            pspeed: pspeed,
            curr_packet: None,
            bits_processed: 0.0,
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

    // Server.process checks to see if a packet is currently being processed, and if so,
    // increments Server.bits_processed, and if the resulting sum is equal to the bits
    // in the packet, then it returns the packet and resets the state of Server.

    pub fn process(&mut self) -> Option<Packet> {
        match self.curr_packet.clone() {
            Some(p) => {
                self.bits_processed += self.pspeed / self.resolution;
                if self.bits_processed as u32 == p.length {
                    self.curr_packet = None;
                    self.bits_processed = 0.0;
                    self.statistics.packets_processed += 1;
                    Some(p)
                } else {
                    None
                }
            }
            None => {
                if let Some(p) = self.queue.pop_front() {
                    self.curr_packet = Some(p.clone());
                    self.bits_processed += self.pspeed / self.resolution;
                    if self.bits_processed as u32 == p.length {
                        self.curr_packet = None;
                        self.bits_processed = 0.0;
                        self.statistics.packets_processed += 1;
                        return Some(p);
                    }
                } else {
                    self.statistics.idle_count += 1;
                }
                None
            }
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
        let mut s = Server::new(1.0, 0.5, None);
        s.enqueue(Packet {
            time_generated: 0,
            length: 1,
        });
        s.enqueue(Packet {
            time_generated: 0,
            length: 1,
        });
        s.process();
        s.process();
        assert_eq!(s.statistics.packets_processed, 1);
        s.process();
        s.process();
        assert_eq!(s.statistics.packets_processed, 2);
    }

    #[test]
    fn server_packet_dropped() {
        let mut s = Server::new(1.0, 1.0, Some(1));
        s.enqueue(Packet {
            time_generated: 0,
            length: 1,
        });
        s.enqueue(Packet {
            time_generated: 0,
            length: 1,
        });

        s.process();
        assert_eq!(s.statistics.packets_processed, 1);
        assert_eq!(s.statistics.packets_dropped, 1);
    }

    #[test]
    fn server_idle_count() {
        let mut s = Server::new(1.0, 1.0, Some(1));

        s.process();
        assert_eq!(s.statistics.idle_count, 1);

        s.process();
        assert_eq!(s.statistics.idle_count, 2);

        s.enqueue(Packet {
            time_generated: 0,
            length: 1,
        });
        s.process();
        assert_eq!(s.statistics.idle_count, 2);
        assert_eq!(s.statistics.packets_processed, 1);
    }
}
