use generators::Generator;

// Packet holds the value of the time in ticks that it was generated at.
struct Packet(u32);

// Client generates packets according as per the provided generators::Generator.
#[allow(dead_code)]
struct Client<G: Generator> {
    // Last tick that a packet was generated.
    last_gen: u32,
    // Ticks till next packet is generated.
    till_next: u32,
    // The "length" of time of one tick.
    resolution: f64,
    generator: G,
}

impl<G: Generator> Client<G> {
    #[allow(dead_code)]
    fn new(generator: G, resolution: f64) -> Client<G> {
        Client {
            last_gen: 0,
            till_next: 0,
            generator: generator,
            resolution: resolution,
        }
    }

    // next_packet takes the current time and uses it to determine whether to generate a packet or
    // not. If a packet is generated, the last_gen time and till_next times are both updated.
    #[allow(dead_code)]
    fn next_packet(&mut self, time: u32) -> Option<Packet> {
        if time - self.last_gen == self.till_next {
            self.last_gen = time;
            self.till_next = self.generator.next_event(self.resolution);
            Some(Packet(time))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::generators::Generator;

    #[test]
    fn test_packet_generator() {
        struct TestGenerator(u32);

        impl Generator for TestGenerator {
            fn next_event(&self, _: f64) -> u32 {
                self.0
            }
        }

        let mut pg = Client::new(TestGenerator(5), 1e6);

        assert_eq!(pg.next_packet(0).expect("invalid value").0, 0);
        assert_eq!(pg.last_gen, 0);
        assert_eq!(pg.till_next, 5);
        assert!(pg.next_packet(3).is_none());
        assert_eq!(pg.last_gen, 0);
        assert_eq!(pg.till_next, 5);
    }
}
