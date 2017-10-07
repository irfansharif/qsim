extern crate rand;

use rand::distributions::{IndependentSample, Range};

trait TimeGenerator {
    fn generate(&self) -> u32;
}

/* MarkovianGenerator generates random numbers that fit an exponential distribution */
struct MarkovianGenerator {
    lambda: f64,
    range: Range<f64>,
}

impl MarkovianGenerator {
    fn new(lambda: f64) -> MarkovianGenerator {
        let range: Range<f64> = Range::new(0.0, 1.0);
        MarkovianGenerator {
            lambda: lambda,
            range: range,
        }
    }
}

impl TimeGenerator for MarkovianGenerator {
    fn generate(&self) -> u32 {
        let mut rng = rand::thread_rng();
        let u = self.range.ind_sample(&mut rng);
        // Use a tick duration of 1 microsecond like in lab manual example
        ((-1.0 / self.lambda) * (1.0 - u).ln() * 1_000_000.0).trunc() as u32
    }
}

// Packet holds the value of the time in ticks that it was generated at
struct Packet(u32);

// PacketGenerator generates packets according to an AvailabilityTimeGenerator
struct PacketGenerator<'a> {
    // Last tick that a packet was generated
    last_gen: u32,
    // Ticks till next packet is generated
    till_next: u32,
    tg: &'a TimeGenerator,
}

impl<'a> PacketGenerator<'a> {
    fn new(tg: &TimeGenerator) -> PacketGenerator {
        PacketGenerator {
            last_gen: 0,
            till_next: 0,
            tg: tg,
        }
    }

    // generate takes the current time and uses it to determine whether to generate a packet or not.
    // if a packet is generated, the last_gen time and till_next times are both updated
    fn generate(&mut self, time: u32) -> Option<Packet> {
        if time - self.last_gen == self.till_next {
            self.last_gen = time;
            self.till_next = self.tg.generate();
            Some(Packet(time))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Just testing to check that the numbers generated seem to be random... use
    // cargo test -- --nocapture to see stdout
    #[test]
    fn print_random_numbers() {
        let mg = MarkovianGenerator::new(100.0);
        for i in 0..10 {
            println!("{}", mg.generate())
        }
    }

    struct TestGenerator(u32);

    impl TimeGenerator for TestGenerator {
        fn generate(&self) -> u32 {
            self.0
        }
    }

    #[test]
    fn test_packet_generator() {
        let tg = TestGenerator(5);
        let mut pg = PacketGenerator::new(&tg);
        let mut packet = pg.generate(0);
        assert_eq!(packet.expect("invalid value").0, 0);
        assert_eq!(pg.last_gen, 0);
        assert_eq!(pg.till_next, 5);
        packet = pg.generate(3);
        assert!(packet.is_none());
        assert_eq!(pg.last_gen, 0);
        assert_eq!(pg.till_next, 5);
    }
}
